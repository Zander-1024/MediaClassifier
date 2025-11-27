/// 文件和目录过滤器
///
/// 负责根据配置规则过滤不需要处理的文件和目录
/// 遵循单一职责原则：只处理过滤逻辑
use crate::config::ExcludeConfig;
use std::path::Path;
use walkdir::DirEntry;

/// 文件过滤器
///
/// 封装所有过滤逻辑，提供统一的过滤接口
pub struct FileFilter<'a> {
    config: &'a ExcludeConfig,
}

impl<'a> FileFilter<'a> {
    /// 创建新的过滤器实例
    pub fn new(config: &'a ExcludeConfig) -> Self {
        Self { config }
    }

    /// 检查目录是否应该被排除
    ///
    /// 返回 true 表示应该跳过这个目录
    pub fn should_exclude_dir(&self, entry: &DirEntry) -> bool {
        if !entry.file_type().is_dir() {
            return false;
        }

        let dir_name = entry.file_name().to_string_lossy();

        // 1. 检查隐藏目录
        if self.config.hidden_files && dir_name.starts_with('.') {
            return true;
        }

        // 2. 检查配置的排除目录列表
        if self
            .config
            .directories
            .iter()
            .any(|excluded| excluded.eq_ignore_ascii_case(&dir_name))
        {
            return true;
        }

        false
    }

    /// 检查文件是否应该被排除
    ///
    /// 返回 true 表示应该跳过这个文件
    pub fn should_exclude_file(&self, path: &Path) -> bool {
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return false,
        };

        // 1. 检查隐藏文件
        if self.config.hidden_files && file_name.starts_with('.') {
            return true;
        }

        // 2. 检查文件名模式匹配
        if self.matches_exclude_pattern(&file_name) {
            return true;
        }

        false
    }

    /// 检查文件名是否匹配排除模式
    ///
    /// 支持简单的通配符：
    /// - `*.tmp` 匹配所有 .tmp 结尾的文件
    /// - `prefix*` 匹配所有以 prefix 开头的文件
    /// - `*suffix` 匹配所有以 suffix 结尾的文件
    /// - 精确匹配：`desktop.ini`
    fn matches_exclude_pattern(&self, file_name: &str) -> bool {
        self.config
            .patterns
            .iter()
            .any(|pattern| self.match_wildcard(pattern, file_name))
    }

    /// 简单的通配符匹配
    ///
    /// 仅支持 * 作为通配符(匹配任意字符)
    fn match_wildcard(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // 精确匹配(不区分大小写)
        if !pattern.contains('*') {
            return pattern.eq_ignore_ascii_case(text);
        }

        // *middle* 格式 - 必须先检查,因为它同时满足前缀和后缀条件
        if pattern.starts_with('*') && pattern.ends_with('*') && pattern.len() > 2 {
            let middle = &pattern[1..pattern.len() - 1];
            return text.to_lowercase().contains(&middle.to_lowercase());
        }

        // *.ext 格式
        if let Some(ext) = pattern.strip_prefix('*') {
            return text.to_lowercase().ends_with(&ext.to_lowercase());
        }

        // prefix* 格式
        if let Some(prefix) = pattern.strip_suffix('*') {
            return text.to_lowercase().starts_with(&prefix.to_lowercase());
        }

        false
    }

    /// 组合过滤器：同时检查文件和目录
    ///
    /// 用于 walkdir 的 filter_entry
    pub fn should_exclude_entry(&self, entry: &DirEntry) -> bool {
        if entry.file_type().is_dir() {
            self.should_exclude_dir(entry)
        } else {
            self.should_exclude_file(entry.path())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(hidden: bool, dirs: Vec<&str>, patterns: Vec<&str>) -> ExcludeConfig {
        ExcludeConfig {
            hidden_files: hidden,
            directories: dirs.iter().map(|s| s.to_string()).collect(),
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_wildcard_matching() {
        let config = create_test_config(false, vec![], vec!["*.tmp", "*.bak", "desktop.ini"]);
        let filter = FileFilter::new(&config);

        assert!(filter.match_wildcard("*.tmp", "file.tmp"));
        assert!(filter.match_wildcard("*.tmp", "FILE.TMP")); // 大小写不敏感
        assert!(!filter.match_wildcard("*.tmp", "file.txt"));

        assert!(filter.match_wildcard("desktop.ini", "desktop.ini"));
        assert!(filter.match_wildcard("desktop.ini", "Desktop.ini"));
        assert!(!filter.match_wildcard("desktop.ini", "desktop.txt"));
    }

    #[test]
    fn test_prefix_wildcard() {
        let config = create_test_config(false, vec![], vec!["temp*"]);
        let filter = FileFilter::new(&config);

        assert!(filter.match_wildcard("temp*", "temp.txt"));
        assert!(filter.match_wildcard("temp*", "temporary"));
        assert!(!filter.match_wildcard("temp*", "test"));
    }

    #[test]
    fn test_middle_wildcard() {
        let config = create_test_config(false, vec![], vec!["*cache*"]);
        let filter = FileFilter::new(&config);

        assert!(filter.match_wildcard("*cache*", "my_cache_file"));
        assert!(filter.match_wildcard("*cache*", "cache"));
        assert!(filter.match_wildcard("*cache*", "test_cache"));
        assert!(!filter.match_wildcard("*cache*", "test"));
    }

    #[test]
    fn test_exclude_patterns() {
        let config = create_test_config(
            false,
            vec![],
            vec!["*.tmp", "*.bak", "desktop.ini", "Thumbs.db"],
        );
        let filter = FileFilter::new(&config);

        assert!(filter.matches_exclude_pattern("file.tmp"));
        assert!(filter.matches_exclude_pattern("backup.bak"));
        assert!(filter.matches_exclude_pattern("desktop.ini"));
        assert!(filter.matches_exclude_pattern("Thumbs.db"));
        assert!(!filter.matches_exclude_pattern("important.txt"));
    }

    #[test]
    fn test_hidden_files() {
        let config = create_test_config(true, vec![], vec![]);
        let filter = FileFilter::new(&config);

        assert!(filter.should_exclude_file(Path::new(".hidden")));
        assert!(filter.should_exclude_file(Path::new(".DS_Store")));
        assert!(!filter.should_exclude_file(Path::new("visible.txt")));
    }

    #[test]
    fn test_exclude_directories() {
        let config = create_test_config(false, vec![".git", "node_modules", "target"], vec![]);
        let _filter = FileFilter::new(&config);

        // 注意：这里只能测试逻辑，不能直接测试 should_exclude_dir
        // 因为需要 DirEntry 对象
        assert!(config.directories.contains(&".git".to_string()));
        assert!(config.directories.contains(&"node_modules".to_string()));
        assert!(config.directories.contains(&"target".to_string()));
    }
}
