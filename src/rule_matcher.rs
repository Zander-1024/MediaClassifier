use crate::config::{Config, FileSize, Rule};
use crate::media_types::MediaType;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::path::Path;

/// 规则匹配器
pub struct RuleMatcher<'a> {
    config: &'a Config,
}

impl<'a> RuleMatcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// 为给定的文件找到匹配的规则
    #[allow(dead_code)]
    pub fn find_matching_rule(&self, extension: &str, file_size: u64) -> Option<&Rule> {
        self.match_file(extension, file_size)
    }

    /// 匹配文件（别名方法）
    pub fn match_file(&self, extension: &str, file_size: u64) -> Option<&Rule> {
        let file_size_obj = FileSize { bytes: file_size };

        for rule in &self.config.rules {
            if !rule.enabled {
                continue;
            }

            // 检查扩展名匹配
            if !self.matches_extension(rule, extension) {
                continue;
            }

            // 检查文件大小（如果规则指定了文件大小限制）
            if let Some(filter) = &rule.file_size {
                let min = if let Some(min_str) = &filter.min {
                    match FileSize::parse(min_str) {
                        Ok(size) => size,
                        Err(_) => continue, // 解析失败，跳过这个规则
                    }
                } else {
                    FileSize { bytes: 0 }
                };

                let max = if let Some(max_str) = &filter.max {
                    match FileSize::parse(max_str) {
                        Ok(size) => size,
                        Err(_) => continue, // 解析失败，跳过这个规则
                    }
                } else {
                    FileSize { bytes: 0 }
                };

                if !file_size_obj.is_in_range(min, max) {
                    continue;
                }
            }

            // 找到匹配的规则
            return Some(rule);
        }

        None
    }

    /// 检查扩展名是否匹配规则
    fn matches_extension(&self, rule: &Rule, extension: &str) -> bool {
        let ext_lower = extension.to_lowercase();

        for rule_ext in &rule.extensions {
            if rule_ext == "*" {
                return true;
            }

            let rule_ext_lower = rule_ext.to_lowercase();

            // 直接匹配
            if rule_ext_lower == ext_lower {
                return true;
            }

            // 检查规则扩展名是否是某个别名的一部分
            // 例如：rule_ext = "jpg", aliases["JPG"] = ["jpg", "jpeg"]
            // 如果文件是 "jpeg"，应该也能匹配
            for alias_exts in self.config.extension_aliases.values() {
                if alias_exts
                    .iter()
                    .any(|a| a.to_lowercase() == rule_ext_lower)
                {
                    // rule_ext 是某个别名组的一部分
                    if alias_exts.iter().any(|a| a.to_lowercase() == ext_lower) {
                        // 文件扩展名也在同一个别名组中
                        return true;
                    }
                }
            }
        }

        false
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
