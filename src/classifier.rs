use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use log::{error, info, warn};
use std::path::{Path, PathBuf};

use crate::conflict::{ConflictResolution, resolve_conflict};
use crate::media_types::{MediaInfo, MediaType, get_media_info};
use crate::metadata::{extract_date, format_date};

/// 文件分类结果
#[derive(Debug)]
pub enum ClassifyResult {
    /// 成功移动文件
    Success { from: PathBuf, to: PathBuf },
    /// 跳过文件（已存在且相同）
    Skipped { path: PathBuf },
    /// 重命名后移动
    Renamed { from: PathBuf, to: PathBuf },
    /// 失败
    Failed { path: PathBuf, error: String },
}

/// 分类单个文件
pub fn classify_file(source: &Path) -> Result<ClassifyResult> {
    // 1. 获取媒体信息
    let media_info = match get_media_info(source) {
        Some(info) => info,
        None => {
            return Ok(ClassifyResult::Failed {
                path: source.to_path_buf(),
                error: "Not a media file".to_string(),
            });
        },
    };

    // 2. 提取日期
    let is_image = media_info.media_type == MediaType::Image;
    let date = match extract_date(source, is_image) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to extract date from {:?}: {}", source, e);
            return Ok(ClassifyResult::Failed {
                path: source.to_path_buf(),
                error: format!("Failed to extract date: {}", e),
            });
        },
    };

    // 3. 构建目标路径
    let target = build_target_path(source, &media_info, &date)?;

    // 4. 检查是否是分类目录中的文件（避免重复处理）
    if is_classified_file(source) {
        return Ok(ClassifyResult::Skipped {
            path: source.to_path_buf(),
        });
    }

    // 5. 解决冲突
    match resolve_conflict(source, &target)? {
        ConflictResolution::NoConflict(final_target) => {
            // 无冲突，直接移动
            move_file(source, &final_target)?;
            info!("Successfully moved: {:?} → {:?}", source, final_target);
            Ok(ClassifyResult::Success {
                from: source.to_path_buf(),
                to: final_target,
            })
        },
        ConflictResolution::Skip(reason) => {
            // 文件相同，跳过
            info!("Skipped: {:?} - {}", source, reason);
            Ok(ClassifyResult::Skipped {
                path: source.to_path_buf(),
            })
        },
        ConflictResolution::Rename(new_target) => {
            // 文件不同，重命名后移动
            move_file(source, &new_target)?;
            warn!(
                "File renamed due to conflict: {:?} → {:?}",
                source, new_target
            );
            Ok(ClassifyResult::Renamed {
                from: source.to_path_buf(),
                to: new_target,
            })
        },
    }
}

/// 构建目标路径：类型/YYYYMMDD/文件名
fn build_target_path(
    source: &Path,
    media_info: &MediaInfo,
    date: &DateTime<Local>,
) -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let date_str = format_date(date);
    let filename = source.file_name().context("Failed to get filename")?;

    // 构建路径：当前目录/扩展名/日期/文件名
    let target = current_dir
        .join(&media_info.extension)
        .join(date_str)
        .join(filename);

    Ok(target)
}

/// 检查文件是否已经在分类目录中
/// 分类目录的特征：路径中包含 扩展名/日期/ 的模式
fn is_classified_file(path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if let Some(date_dir) = parent.file_name() {
            // 检查父目录名是否是日期格式（8位数字）
            let date_str = date_dir.to_string_lossy();
            if date_str.len() == 8 && date_str.chars().all(|c| c.is_ascii_digit()) {
                // 检查祖父目录是否是扩展名（全大写字母）
                if let Some(grandparent) = parent.parent() {
                    if let Some(ext_dir) = grandparent.file_name() {
                        let ext_str = ext_dir.to_string_lossy();
                        if ext_str
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// 移动文件到目标位置
fn move_file(source: &Path, target: &Path) -> Result<()> {
    // 确保目标目录存在
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("Failed to create target directory")?;
    }

    // 移动文件
    std::fs::rename(source, target).context("Failed to move file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_classified_file() {
        let path1 = Path::new("/home/user/JPG/20251118/photo.jpg");
        assert!(is_classified_file(path1));

        let path2 = Path::new("/home/user/photos/photo.jpg");
        assert!(!is_classified_file(path2));

        let path3 = Path::new("/home/user/NEF/20251115/DSC_1234.NEF");
        assert!(is_classified_file(path3));
    }
}
