mod classifier;
mod conflict;
mod media_types;
mod metadata;

use anyhow::{Context, Result};
use classifier::{ClassifyResult, classify_file};
use log::{error, info};
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 统计信息
#[derive(Debug, Default)]
struct Statistics {
    success: usize,
    skipped: usize,
    renamed: usize,
    failed: usize,
}

impl Statistics {
    fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, result: ClassifyResult) {
        match result {
            ClassifyResult::Success { .. } => self.success += 1,
            ClassifyResult::Skipped { .. } => self.skipped += 1,
            ClassifyResult::Renamed { .. } => self.renamed += 1,
            ClassifyResult::Failed { .. } => self.failed += 1,
        }
    }

    fn print_summary(&self) {
        println!("\n========== Classification Summary ==========");
        println!("✅ Successfully moved:  {}", self.success);
        println!("🔄 Renamed and moved:   {}", self.renamed);
        println!("⏭️  Skipped (same file): {}", self.skipped);
        println!("❌ Failed:              {}", self.failed);
        println!("📊 Total processed:     {}", self.total());
        println!("==========================================\n");

        info!(
            "Classification completed: {} success, {} renamed, {} skipped, {} failed",
            self.success, self.renamed, self.skipped, self.failed
        );
    }

    fn total(&self) -> usize {
        self.success + self.skipped + self.renamed + self.failed
    }
}

fn main() -> Result<()> {
    // 初始化日志系统
    init_logger()?;

    info!("MediaClassifier started");
    println!("🚀 MediaClassifier - Organizing your media files...\n");

    // 获取当前目录
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    info!("Working directory: {:?}", current_dir);
    println!("📁 Working directory: {}\n", current_dir.display());

    // 扫描并收集所有媒体文件
    println!("🔍 Scanning for media files...");
    let media_files = scan_media_files(&current_dir)?;

    if media_files.is_empty() {
        println!("ℹ️  No media files found in the current directory.");
        info!("No media files found");
        return Ok(());
    }

    println!("📋 Found {} media files\n", media_files.len());
    info!("Found {} media files", media_files.len());

    // 处理每个文件
    println!("⚙️  Processing files...\n");
    let mut stats = Statistics::new();

    for (index, file) in media_files.iter().enumerate() {
        let progress = format!("[{}/{}]", index + 1, media_files.len());

        match classify_file(file) {
            Ok(result) => {
                match &result {
                    ClassifyResult::Success { from, to } => {
                        println!(
                            "{} ✅ Moved: {} → {}",
                            progress,
                            from.file_name().unwrap().to_string_lossy(),
                            to.strip_prefix(&current_dir).unwrap_or(to).display()
                        );
                    },
                    ClassifyResult::Renamed { from, to, .. } => {
                        println!(
                            "{} 🔄 Renamed: {} → {}",
                            progress,
                            from.file_name().unwrap().to_string_lossy(),
                            to.strip_prefix(&current_dir).unwrap_or(to).display()
                        );
                    },
                    ClassifyResult::Skipped { path, .. } => {
                        println!(
                            "{} ⏭️  Skipped: {} (already exists)",
                            progress,
                            path.file_name().unwrap().to_string_lossy()
                        );
                    },
                    ClassifyResult::Failed { path, error } => {
                        println!(
                            "{} ❌ Failed: {} - {}",
                            progress,
                            path.file_name().unwrap().to_string_lossy(),
                            error
                        );
                    },
                }
                stats.record(result);
            },
            Err(e) => {
                error!("Error processing {:?}: {}", file, e);
                println!(
                    "{} ❌ Error: {} - {}",
                    progress,
                    file.file_name().unwrap().to_string_lossy(),
                    e
                );
                stats.failed += 1;
            },
        }
    }

    // 打印统计信息
    stats.print_summary();

    println!("📝 Detailed logs saved to: classifier.log");
    println!("✨ Done!\n");

    Ok(())
}

/// 初始化日志系统
fn init_logger() -> Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("classifier.log").context("Failed to create log file")?,
    )])
    .context("Failed to initialize logger")?;

    Ok(())
}

/// 扫描目录中的所有媒体文件
fn scan_media_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut media_files = Vec::new();

    for entry in WalkDir::new(dir)
        .min_depth(1) // 跳过根目录本身
        .max_depth(3) // 限制递归深度，避免扫描太深
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && !is_target_dir(e))
    {
        let entry = entry.context("Failed to read directory entry")?;

        // 只处理文件
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // 检查是否为媒体文件
        if media_types::get_media_info(path).is_some() {
            media_files.push(path.to_path_buf());
        }
    }

    Ok(media_files)
}

/// 检查是否为隐藏文件/目录
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// 检查是否为应该跳过的目录
fn is_target_dir(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    let name = entry.file_name().to_string_lossy();

    // 跳过 target 目录（Rust 编译输出）
    if name == "target" {
        return true;
    }

    // 跳过看起来像是分类目录的目录（全大写字母）
    if name.len() <= 5
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return true;
    }

    false
}
