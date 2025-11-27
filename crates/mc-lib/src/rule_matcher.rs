use crate::config::{Config, FileSize, Rule};
use crate::media_types::MediaType;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::path::Path;

/// 规则匹配器
pub struct RuleMatcher<'a> {
    config: &'a Config,
    /// 扩展名到规则列表的映射 (支持同一扩展名多个规则，按优先级排序)
    extension_rules: HashMap<String, Vec<&'a Rule>>,
    /// 全局默认规则（用于已知媒体类型但未特别配置的文件）
    global_rule: Option<&'a Rule>,
}

impl<'a> RuleMatcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        let mut extension_rules: HashMap<String, Vec<&'a Rule>> = HashMap::new();
        let mut global_rule = None;

        // 构建扩展名到规则的索引映射
        for rule in &config.rules {
            if !rule.enabled {
                continue;
            }

            // 识别全局默认规则 (名称包含 "Default" 且没有具体扩展名)
            if rule.name.to_lowercase().contains("default")
                && (rule.extensions.is_empty()
                    || (rule.extensions.len() == 1
                        && rule.extensions[0] == "*"
                        && global_rule.is_none()))
            {
                // 如果还没有 global_rule，或者这个规则明确不是通配符
                if rule.extensions.is_empty() {
                    global_rule = Some(rule);
                    continue;
                }
            }

            // 为每个扩展名建立索引
            for ext in &rule.extensions {
                let ext_lower = ext.to_lowercase();

                // 直接添加该扩展名
                extension_rules
                    .entry(ext_lower.clone())
                    .or_default()
                    .push(rule);

                // 处理别名：将别名组中的所有扩展名都映射到这个规则
                for alias_exts in config.extension_aliases.values() {
                    if alias_exts.iter().any(|a| a.eq_ignore_ascii_case(ext)) {
                        // 当前扩展名是某个别名组的一部分
                        for alias_ext in alias_exts {
                            let alias_lower = alias_ext.to_lowercase();
                            // 避免重复添加
                            let rules_vec = extension_rules.entry(alias_lower).or_default();

                            if !rules_vec.iter().any(|r| std::ptr::eq(*r, rule)) {
                                rules_vec.push(rule);
                            }
                        }
                        break; // 找到别名组后跳出
                    }
                }
            }
        }

        Self {
            config,
            extension_rules,
            global_rule,
        }
    }

    /// 为给定的文件找到匹配的规则
    #[allow(dead_code)]
    pub fn find_matching_rule(&self, extension: &str, file_size: u64) -> Option<&Rule> {
        self.match_file(extension, file_size)
    }

    /// 匹配文件（O(1) 优化版本）
    pub fn match_file(&self, extension: &str, file_size: u64) -> Option<&Rule> {
        let ext_lower = extension.to_lowercase();
        let file_size_obj = FileSize { bytes: file_size };

        // 1. O(1) 查找：从扩展名索引中查找匹配的规则列表
        if let Some(rules) = self.extension_rules.get(&ext_lower) {
            // 按规则顺序检查文件大小匹配（规则已按配置文件中的顺序排列，即优先级）
            for &rule in rules {
                if self.check_file_size_match(rule, &file_size_obj) {
                    return Some(rule);
                }
            }
        }

        // 2. 如果是已知的媒体类型但未在规则中配置，使用全局默认规则
        if self.is_supported_media_type(&ext_lower) && self.global_rule.is_some() {
            return self.global_rule;
        }
        None
    }

    /// 检查文件大小是否匹配规则
    fn check_file_size_match(&self, rule: &Rule, file_size: &FileSize) -> bool {
        if let Some(filter) = &rule.file_size {
            let min = if let Some(min_str) = &filter.min {
                match FileSize::parse(min_str) {
                    Ok(size) => size,
                    Err(_) => return false, // 解析失败，不匹配
                }
            } else {
                FileSize { bytes: 0 }
            };

            let max = if let Some(max_str) = &filter.max {
                match FileSize::parse(max_str) {
                    Ok(size) => size,
                    Err(_) => return false, // 解析失败，不匹配
                }
            } else {
                FileSize { bytes: 0 }
            };

            return file_size.is_in_range(min, max);
        }

        true // 没有文件大小限制，匹配
    }

    /// 检查扩展名是否为已知的媒体类型
    fn is_supported_media_type(&self, ext: &str) -> bool {
        crate::media_types::is_image_extension(ext)
            || crate::media_types::is_video_extension(ext)
            || crate::media_types::is_audio_extension(ext)
    }

    /// 根据规则构建目标路径
    pub fn build_target_path(
        &self,
        base_dir: &Path,
        source: &Path,
        media_info: &crate::media_types::MediaInfo,
        date: Option<&DateTime<Local>>,
        rule: &Rule,
    ) -> Result<std::path::PathBuf> {
        let filename = source.file_name().context("Failed to get filename")?;

        // 如果规则不需要日期，使用简单模板
        if date.is_none() || rule.date_format.is_none() {
            let mut template = rule.directory_template.clone();
            template = template.replace("{ext}", &media_info.extension);
            template = template.replace(
                "{type}",
                match media_info.media_type {
                    MediaType::Image => "Image",
                    MediaType::Video => "Video",
                    MediaType::Audio => "Audio",
                },
            );
            return Ok(base_dir.join(template).join(filename));
        }

        let date = date.unwrap();
        self.build_path(
            rule,
            base_dir,
            &media_info.extension,
            media_info.media_type.clone(),
            date,
            filename.to_str().unwrap(),
        )
    }

    /// 根据规则构建目标路径（内部方法）
    fn build_path(
        &self,
        rule: &Rule,
        base_dir: &Path,
        extension: &str,
        media_type: MediaType,
        date: &DateTime<Local>,
        filename: &str,
    ) -> Result<std::path::PathBuf> {
        let template = &rule.directory_template;
        let date_format = rule
            .date_format
            .as_deref()
            .unwrap_or(&self.config.global.date_format);

        let expanded = expand_template(template, extension, media_type, date, date_format)?;

        Ok(base_dir.join(expanded).join(filename))
    }
}

/// 展开模板变量
fn expand_template(
    template: &str,
    extension: &str,
    media_type: MediaType,
    date: &DateTime<Local>,
    date_format: &str,
) -> Result<String> {
    let mut result = template.to_string();

    // {type}
    let type_str = match media_type {
        MediaType::Image => "Image",
        MediaType::Video => "Video",
        MediaType::Audio => "Audio",
    };
    result = result.replace("{type}", type_str);

    // {ext}
    result = result.replace("{ext}", &extension.to_uppercase());

    // {year}
    result = result.replace("{year}", &date.format("%Y").to_string());

    // {month}
    result = result.replace("{month}", &date.format("%m").to_string());

    // {day}
    result = result.replace("{day}", &date.format("%d").to_string());

    // {date} - 根据 date_format 格式化
    let formatted_date = format_date_string(date, date_format)?;
    result = result.replace("{date}", &formatted_date);

    Ok(result)
}

/// 根据日期格式字符串格式化日期
fn format_date_string(date: &DateTime<Local>, format: &str) -> Result<String> {
    let format = format.trim();

    let result = match format {
        "YYYY" => date.format("%Y").to_string(),
        "YYYYMM" => date.format("%Y%m").to_string(),
        "YYYYMMDD" => date.format("%Y%m%d").to_string(),
        "YYYY/MMDD" => date.format("%Y/%m%d").to_string(),
        "YYYY/MM" => date.format("%Y/%m").to_string(),
        "YYYY/MM/DD" => date.format("%Y/%m/%d").to_string(),
        "YYYY-MM" => date.format("%Y-%m").to_string(),
        "YYYY-MM-DD" => date.format("%Y-%m-%d").to_string(),
        // 允许自定义格式
        custom => date.format(custom).to_string(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_date_string() {
        let date = Local.with_ymd_and_hms(2025, 11, 18, 14, 30, 0).unwrap();

        assert_eq!(format_date_string(&date, "YYYY").unwrap(), "2025");
        assert_eq!(format_date_string(&date, "YYYYMM").unwrap(), "202511");
        assert_eq!(format_date_string(&date, "YYYYMMDD").unwrap(), "20251118");
        assert_eq!(format_date_string(&date, "YYYY/MM").unwrap(), "2025/11");
        assert_eq!(
            format_date_string(&date, "YYYY/MM/DD").unwrap(),
            "2025/11/18"
        );
        assert_eq!(
            format_date_string(&date, "YYYY-MM-DD").unwrap(),
            "2025-11-18"
        );
    }

    #[test]
    fn test_expand_template() {
        let date = Local.with_ymd_and_hms(2025, 11, 18, 14, 30, 0).unwrap();

        let result =
            expand_template("{ext}/{date}", "jpg", MediaType::Image, &date, "YYYYMMDD").unwrap();
        assert_eq!(result, "JPG/20251118");

        let result = expand_template(
            "Photos/{year}/{month}",
            "jpg",
            MediaType::Image,
            &date,
            "YYYYMMDD",
        )
        .unwrap();
        assert_eq!(result, "Photos/2025/11");

        let result =
            expand_template("{type}/{year}", "mp4", MediaType::Video, &date, "YYYY").unwrap();
        assert_eq!(result, "Video/2025");
    }

    #[test]
    fn test_rule_matcher() {
        let config = Config::default();
        let matcher = RuleMatcher::new(&config);

        // 测试大文件 JPG（应该匹配 High Quality Photos 规则）
        let rule = matcher.find_matching_rule("jpg", 10 * 1024 * 1024); // 10MB
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "High Quality Photos");

        // 测试小文件 JPG（应该匹配 Thumbnails 规则）
        let rule = matcher.find_matching_rule("jpg", 1024 * 1024); // 1MB
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "Thumbnails");

        // 测试 NEF（应该匹配 RAW Photos 规则）
        let rule = matcher.find_matching_rule("nef", 20 * 1024 * 1024); // 20MB
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "RAW Photos");

        // 测试 MP4（应该匹配 Videos 规则）
        let rule = matcher.find_matching_rule("mp4", 100 * 1024 * 1024); // 100MB
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "Videos");
    }
}
