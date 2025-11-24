use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 配置文件根结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub extension_aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub exclude: ExcludeConfig,
}

/// 全局配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub date_format: String,
    pub directory_template: String,
    pub clean_empty_dirs: bool,
    pub file_size: FileSizeFilter,
}

/// 文件分类规则
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub file_size: FileSizeFilter,
    pub directory_template: String,
    pub date_format: Option<String>,
    pub enabled: bool,
}

/// 文件大小过滤器
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileSizeFilter {
    /// 最小文件大小，如 "5MB", "100KB"
    pub min: String,
    /// 最大文件大小，如 "1GB"
    pub max: String,
}

/// 排除规则配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExcludeConfig {
    #[serde(default = "default_true")]
    pub hidden_files: bool,
    #[serde(default)]
    pub directories: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// 解析后的文件大小（以字节为单位）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSize {
    pub bytes: u64,
}

impl FileSize {
    /// 解析文件大小字符串
    /// 支持：B, KB, MB, GB, TB (Byte)
    /// b, Kb, Mb, Gb, Tb (bit, 1 Byte = 8 bits)
    pub fn parse(size_str: &str) -> Result<Self> {
        let size_str = size_str.trim();

        if size_str == "0" || size_str == "0B" || size_str == "0b" {
            return Ok(FileSize { bytes: 0 });
        }

        // 正则匹配数字和单位
        let re = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*([A-Za-z]*)$")?;

        let caps = re
            .captures(size_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid size format: {}", size_str))?;

        let number: f64 = caps[1].parse().context("Invalid number in size string")?;

        let unit = caps.get(2).map_or("B", |m| m.as_str());

        let multiplier: u64 = match unit {
            "B" | "" => 1,
            "KB" => 1024,
            "MB" => 1024 * 1024,
            "GB" => 1024 * 1024 * 1024,
            "TB" => 1024 * 1024 * 1024 * 1024,
            // bit 单位: 8 bits = 1 byte
            "b" => 1,                         // 单个 bit，需要除以8（在计算bytes时处理）
            "Kb" | "kb" => 128,               // 1 kilobit = 1024 bits / 8 = 128 bytes
            "Mb" | "mb" => 128 * 1024,        // megabit
            "Gb" | "gb" => 128 * 1024 * 1024, // gigabit
            "Tb" | "tb" => 128 * 1024 * 1024 * 1024, // terabit
            _ => anyhow::bail!("Unknown unit: {}", unit),
        };

        let bytes = if unit == "b" {
            // 对于单个bit，需要除以8
            (number / 8.0) as u64
        } else {
            (number * multiplier as f64) as u64
        };
        Ok(FileSize { bytes })
    }

    /// 检查文件大小是否在范围内
    pub fn is_in_range(&self, min: FileSize, max: FileSize) -> bool {
        let min_ok = min.bytes == 0 || self.bytes >= min.bytes;
        let max_ok = max.bytes == 0 || self.bytes <= max.bytes;
        min_ok && max_ok
    }

    /// 格式化为人类可读的字符串
    pub fn format(&self) -> String {
        if self.bytes == 0 {
            return "∞".to_string();
        }

        const KB: u64 = 1024;
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * 1024 * 1024;
        const TB: u64 = 1024 * 1024 * 1024 * 1024;

        if self.bytes >= TB {
            format!("{:.2}TB", self.bytes as f64 / TB as f64)
        } else if self.bytes >= GB {
            format!("{:.2}GB", self.bytes as f64 / GB as f64)
        } else if self.bytes >= MB {
            format!("{:.2}MB", self.bytes as f64 / MB as f64)
        } else if self.bytes >= KB {
            format!("{:.2}KB", self.bytes as f64 / KB as f64)
        } else {
            format!("{}B", self.bytes)
        }
    }
}

impl Config {
    /// 创建默认配置
    #[allow(dead_code)]
    pub fn default_config() -> Self {
        Self::default()
    }
    /// 加载配置文件
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            fs::read_to_string(path).context(format!("Failed to read config file: {:?}", path))?;
        let config: Config =
            serde_yaml_bw::from_str(&content).context("Failed to parse YAML config")?;
        Ok(config)
    }

    /// 保存配置文件
    #[allow(dead_code)]
    pub fn save(&self, path: &Path) -> Result<()> {
        let yaml = serde_yaml_bw::to_string(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, yaml)?;
        Ok(())
    }

    /// 获取默认配置
    pub fn default() -> Self {
        Config {
            global: GlobalConfig {
                date_format: "YYYYMMDD".to_string(),
                directory_template: "{ext}/{date}".to_string(),
                clean_empty_dirs: true,
                file_size: FileSizeFilter {
                    min: "0B".to_string(),
                    max: "0B".to_string(),
                },
            },
            rules: vec![
                Rule {
                    name: "High Quality Photos".to_string(),
                    description: "大尺寸照片按年月分类到 Photos 目录".to_string(),
                    extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string()],
                    file_size: FileSizeFilter {
                        min: "5MB".to_string(),
                        max: "0B".to_string(),
                    },
                    directory_template: "Photos/{year}/{month}".to_string(),
                    date_format: Some("YYYY/MM".to_string()),
                    enabled: true,
                },
                Rule {
                    name: "RAW Photos".to_string(),
                    description: "RAW 格式照片按年/月/日详细分类".to_string(),
                    extensions: vec![
                        "nef".to_string(),
                        "cr2".to_string(),
                        "cr3".to_string(),
                        "arw".to_string(),
                        "dng".to_string(),
                        "orf".to_string(),
                        "raf".to_string(),
                        "rw2".to_string(),
                    ],
                    file_size: FileSizeFilter {
                        min: "0B".to_string(),
                        max: "0B".to_string(),
                    },
                    directory_template: "RAW/{year}/{month}/{day}".to_string(),
                    date_format: Some("YYYY/MM/DD".to_string()),
                    enabled: true,
                },
                Rule {
                    name: "Thumbnails".to_string(),
                    description: "小尺寸图片分类到 Thumbnails 目录".to_string(),
                    extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string()],
                    file_size: FileSizeFilter {
                        min: "0B".to_string(),
                        max: "5MB".to_string(),
                    },
                    directory_template: "Thumbnails/{date}".to_string(),
                    date_format: Some("YYYYMMDD".to_string()),
                    enabled: true,
                },
                Rule {
                    name: "Videos".to_string(),
                    description: "视频文件按年份分类".to_string(),
                    extensions: vec![
                        "mp4".to_string(),
                        "mov".to_string(),
                        "avi".to_string(),
                        "mkv".to_string(),
                        "m4v".to_string(),
                        "wmv".to_string(),
                        "flv".to_string(),
                    ],
                    file_size: FileSizeFilter {
                        min: "0B".to_string(),
                        max: "0B".to_string(),
                    },
                    directory_template: "Videos/{year}".to_string(),
                    date_format: Some("YYYY".to_string()),
                    enabled: true,
                },
                Rule {
                    name: "Music".to_string(),
                    description: "音乐文件按格式分类".to_string(),
                    extensions: vec![
                        "mp3".to_string(),
                        "flac".to_string(),
                        "wav".to_string(),
                        "aac".to_string(),
                        "m4a".to_string(),
                    ],
                    file_size: FileSizeFilter {
                        min: "0B".to_string(),
                        max: "0B".to_string(),
                    },
                    directory_template: "Music/{ext}".to_string(),
                    date_format: None,
                    enabled: true,
                },
                Rule {
                    name: "Default".to_string(),
                    description: "默认规则：扩展名/日期格式".to_string(),
                    extensions: vec!["*".to_string()],
                    file_size: FileSizeFilter {
                        min: "0B".to_string(),
                        max: "0B".to_string(),
                    },
                    directory_template: "{ext}/{date}".to_string(),
                    date_format: Some("YYYYMMDD".to_string()),
                    enabled: true,
                },
            ],
            extension_aliases: {
                let mut aliases = HashMap::new();
                aliases.insert(
                    "JPG".to_string(),
                    vec!["jpg".to_string(), "jpeg".to_string()],
                );
                aliases.insert(
                    "TIFF".to_string(),
                    vec!["tif".to_string(), "tiff".to_string()],
                );
                aliases.insert(
                    "MPEG".to_string(),
                    vec!["mpg".to_string(), "mpeg".to_string(), "mpe".to_string()],
                );
                aliases
            },
            exclude: ExcludeConfig {
                hidden_files: true,
                directories: vec![
                    ".git".to_string(),
                    ".svn".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                    "__pycache__".to_string(),
                ],
                patterns: vec![
                    "*.tmp".to_string(),
                    "*.bak".to_string(),
                    "*.swp".to_string(),
                    "desktop.ini".to_string(),
                    "Thumbs.db".to_string(),
                    ".DS_Store".to_string(),
                ],
            },
        }
    }

    /// 生成带注释的默认配置文件内容
    pub fn generate_default_yaml() -> String {
        let content = include_str!("../default_cfg.yaml");
        content.to_string()
    }

    /// 获取默认配置文件路径
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;
        Ok(config_dir.join("media-classifier").join("config.yaml"))
    }

    /// 确保配置文件存在，如果不存在则创建默认配置
    pub fn ensure_config_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, Self::generate_default_yaml())?;
            println!("✨ Created default config file at: {}", path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_parse_bytes() {
        assert_eq!(FileSize::parse("100B").unwrap().bytes, 100);
        assert_eq!(FileSize::parse("100").unwrap().bytes, 100);
        assert_eq!(FileSize::parse("0B").unwrap().bytes, 0);
        assert_eq!(FileSize::parse("0").unwrap().bytes, 0);
    }

    #[test]
    fn test_file_size_parse_kb_mb_gb() {
        assert_eq!(FileSize::parse("1KB").unwrap().bytes, 1024);
        assert_eq!(FileSize::parse("1MB").unwrap().bytes, 1024 * 1024);
        assert_eq!(FileSize::parse("1GB").unwrap().bytes, 1024 * 1024 * 1024);
        assert_eq!(
            FileSize::parse("1TB").unwrap().bytes,
            1024 * 1024 * 1024 * 1024
        );
    }

    #[test]
    fn test_file_size_parse_bits() {
        assert_eq!(FileSize::parse("8b").unwrap().bytes, 1);
        assert_eq!(FileSize::parse("1Kb").unwrap().bytes, 128);
        assert_eq!(FileSize::parse("1Mb").unwrap().bytes, 128 * 1024);
    }

    #[test]
    fn test_file_size_parse_decimal() {
        let size = FileSize::parse("1.5MB").unwrap().bytes;
        let expected = (1.5 * 1024.0 * 1024.0) as u64;
        assert_eq!(size, expected);
    }

    #[test]
    fn test_file_size_is_in_range() {
        let size = FileSize { bytes: 1024 * 1024 }; // 1MB
        let min = FileSize { bytes: 512 * 1024 }; // 512KB
        let max = FileSize {
            bytes: 2 * 1024 * 1024,
        }; // 2MB

        assert!(size.is_in_range(min, max));

        let no_limit = FileSize { bytes: 0 };
        assert!(size.is_in_range(no_limit, no_limit));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.rules.is_empty());
        assert_eq!(config.global.date_format, "YYYYMMDD");
    }
}
