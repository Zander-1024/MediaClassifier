//! MediaClassifier Core Library
//!
//! 这个库提供了媒体文件分类的核心功能，包括：
//! - 配置管理
//! - 媒体类型识别
//! - 元数据提取
//! - 规则匹配
//! - 文件分类
//!
//! # 使用示例
//!
//! ```no_run
//! use mc_lib::{Config, classify_file_with_config};
//! use std::path::PathBuf;
//!
//! let config = Config::default();
//! let target_dir = PathBuf::from(".");
//! let source = PathBuf::from("photo.jpg");
//!
//! let result = classify_file_with_config(&config, &target_dir, &source);
//! ```

mod classifier;
mod config;
mod conflict;
mod filter;
mod media_types;
mod metadata;
mod rule_matcher;
mod utils;

// Re-export public items
pub use classifier::{ClassifyResult, classify_file, classify_file_with_config};
pub use config::{Config, ExcludeConfig, FileSize, FileSizeFilter, GlobalConfig, Rule};
pub use filter::FileFilter;
pub use media_types::{
    MediaInfo, MediaType, get_media_info, is_audio_extension, is_image_extension,
    is_video_extension,
};
pub use metadata::{extract_date, format_date};
pub use rule_matcher::RuleMatcher;
pub use utils::remove_empty_dirs;
