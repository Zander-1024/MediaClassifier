use anyhow::{Context, Result};
use log::debug;
use std::path::{Path, PathBuf};

/// 文件冲突解决结果
#[derive(Debug)]
pub enum ConflictResolution {
    /// 无冲突，可以直接使用目标路径
    NoConflict(PathBuf),
    /// 文件相同（大小一致），应该跳过
    Skip(String),
    /// 文件不同，需要重命名
    Rename(PathBuf),
}

/// 解决文件冲突
/// 如果目标文件不存在，返回 NoConflict
/// 如果目标文件存在且大小相同，返回 Skip
/// 如果目标文件存在但大小不同，返回 Rename（带新文件名）
pub fn resolve_conflict(source: &Path, target: &Path) -> Result<ConflictResolution> {
    // 如果目标文件不存在，无冲突
    if !target.exists() {
        debug!("No conflict for {:?}", target);
        return Ok(ConflictResolution::NoConflict(target.to_path_buf()));
    }

    // 比较文件大小
    let source_size = std::fs::metadata(source)
        .context("Failed to get source file metadata")?
        .len();

    let target_size = std::fs::metadata(target)
        .context("Failed to get target file metadata")?
        .len();

    if source_size == target_size {
        // 文件大小相同，跳过
        let reason = format!(
            "File already exists with same size ({} bytes): {:?}",
            source_size, target
        );
        debug!("{}", reason);
        Ok(ConflictResolution::Skip(reason))
    } else {
        // 文件大小不同，生成新文件名
        let new_target = generate_unique_name(target)?;
        debug!(
            "File exists with different size (source: {} bytes, target: {} bytes), renaming to {:?}",
            source_size, target_size, new_target
        );
        Ok(ConflictResolution::Rename(new_target))
    }
}

/// 生成唯一的文件名
/// 在文件名后添加数字后缀，如 photo.jpg -> photo_1.jpg
fn generate_unique_name(target: &Path) -> Result<PathBuf> {
    let parent = target.parent().context("Failed to get parent directory")?;
    let file_stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Failed to get file stem")?;
    let extension = target.extension().and_then(|s| s.to_str()).unwrap_or("");

    // 尝试添加数字后缀，直到找到不存在的文件名
    for i in 1..1000 {
        let new_name = if extension.is_empty() {
            format!("{}_{}", file_stem, i)
        } else {
            format!("{}_{}.{}", file_stem, i, extension)
        };

        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return Ok(new_path);
        }
    }

    anyhow::bail!("Failed to generate unique name after 1000 attempts")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_unique_name() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().join("test.jpg");

        // 创建测试文件
        fs::write(&base_path, b"test").unwrap();

        // 生成唯一名称
        let unique = generate_unique_name(&base_path).unwrap();
        assert_eq!(unique.file_name().unwrap(), "test_1.jpg");

        // 创建 test_1.jpg
        fs::write(&unique, b"test").unwrap();

        // 再次生成应该得到 test_2.jpg
        let unique2 = generate_unique_name(&base_path).unwrap();
        assert_eq!(unique2.file_name().unwrap(), "test_2.jpg");
    }
}
