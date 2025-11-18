use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::{debug, warn};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// 从文件中提取日期
/// 图片文件优先使用 EXIF 数据，其他文件使用文件系统时间
pub fn extract_date(path: &Path, is_image: bool) -> Result<DateTime<Local>> {
    if is_image {
        // 尝试从 EXIF 提取日期
        match extract_exif_date(path) {
            Ok(date) => {
                debug!("Extracted EXIF date for {:?}: {}", path, date);
                return Ok(date);
            }
            Err(e) => {
                warn!("Failed to extract EXIF date for {:?}: {}, falling back to file time", path, e);
            }
        }
    }

    // 回退到文件系统时间
    extract_file_date(path)
}

/// 从 EXIF 数据中提取日期
fn extract_exif_date(path: &Path) -> Result<DateTime<Local>> {
    let file = File::open(path).context("Failed to open file for EXIF reading")?;
    let mut bufreader = BufReader::new(&file);
    
    let exifreader = exif::Reader::new();
    let exif = exifreader
        .read_from_container(&mut bufreader)
        .context("Failed to read EXIF data")?;

    // 优先使用 DateTimeOriginal（拍摄时间）
    if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        if let Some(date) = parse_exif_datetime(&field.display_value().to_string()) {
            return Ok(date);
        }
    }

    // 其次使用 DateTime（修改时间）
    if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        if let Some(date) = parse_exif_datetime(&field.display_value().to_string()) {
            return Ok(date);
        }
    }

    anyhow::bail!("No valid date found in EXIF data")
}

/// 解析 EXIF 日期时间字符串
/// EXIF 格式: "2025:11:18 14:30:45" 或 "2025-11-18 14:30:45"
fn parse_exif_datetime(datetime_str: &str) -> Option<DateTime<Local>> {
    // 移除可能的引号
    let datetime_str = datetime_str.trim_matches('"').trim();
    
    // 尝试解析 "YYYY:MM:DD HH:MM:SS" 格式
    if let Ok(naive) = NaiveDateTime::parse_from_str(datetime_str, "%Y:%m:%d %H:%M:%S") {
        return Some(Local.from_local_datetime(&naive).single()?);
    }
    
    // 尝试解析 "YYYY-MM-DD HH:MM:SS" 格式
    if let Ok(naive) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        return Some(Local.from_local_datetime(&naive).single()?);
    }
    
    None
}

/// 从文件系统元数据中提取日期
/// 优先使用创建时间，如果不可用则使用修改时间
fn extract_file_date(path: &Path) -> Result<DateTime<Local>> {
    let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
    
    // 尝试获取创建时间
    if let Ok(created) = metadata.created() {
        let datetime: DateTime<Local> = created.into();
        debug!("Using file creation time for {:?}: {}", path, datetime);
        return Ok(datetime);
    }
    
    // 回退到修改时间
    let modified = metadata.modified().context("Failed to get file modified time")?;
    let datetime: DateTime<Local> = modified.into();
    debug!("Using file modified time for {:?}: {}", path, datetime);
    Ok(datetime)
}

/// 将日期格式化为 YYYYMMDD 格式
pub fn format_date(date: &DateTime<Local>) -> String {
    date.format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exif_datetime() {
        let date1 = parse_exif_datetime("2025:11:18 14:30:45");
        assert!(date1.is_some());
        
        let date2 = parse_exif_datetime("\"2025:11:18 14:30:45\"");
        assert!(date2.is_some());
        
        let date3 = parse_exif_datetime("2025-11-18 14:30:45");
        assert!(date3.is_some());
    }

    #[test]
    fn test_format_date() {
        let datetime_str = "2025:11:18 14:30:45";
        if let Some(date) = parse_exif_datetime(datetime_str) {
            assert_eq!(format_date(&date), "20251118");
        }
    }
}
