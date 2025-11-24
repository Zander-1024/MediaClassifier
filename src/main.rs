mod classifier;
mod config;
mod config_display;
mod conflict;
mod filter;
mod media_types;
mod metadata;
mod rule_matcher;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use classifier::{ClassifyResult, classify_file_with_config};
use config::Config;
use config_display::show_config;
use filter::FileFilter;
use log::{error, info};
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::media_types::is_media_extension;
use crate::utils::remove_empty_dirs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to operate on (default: current directory)
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Config file path (default: ~/.config/media-classifier/config.yaml)
    #[arg(short = 'f', long = "file")]
    config_file: Option<PathBuf>,

    /// Interactive configuration mode
    #[arg(short = 'c', long = "configure")]
    configure: bool,

    /// Show current configuration in table format
    #[arg(short = 's', long = "show-config")]
    show_config: bool,

    /// Remove empty directories after processing (default: from config)
    #[arg(long)]
    clean: Option<bool>,
}

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
    let args = Args::parse();

    // 获取配置文件路径
    let config_path = if let Some(path) = args.config_file {
        path
    } else {
        Config::default_config_path()?
    };

    // 确保配置文件存在
    Config::ensure_config_exists(&config_path)?;

    // 加载配置
    let config = Config::load(&config_path)?;

    // 如果是显示配置模式
    if args.show_config {
        show_config(&config, &config_path.display().to_string());
        return Ok(());
    }

    // 如果是配置模式
    if args.configure {
        println!("🔧 Interactive configuration mode is not yet implemented.");
        println!(
            "📝 Please edit the config file directly: {}",
            config_path.display()
        );
        println!("\nYou can use -s/--show-config to view the current configuration.");
        return Ok(());
    }

    // 初始化日志系统
    init_logger()?;

    info!("MediaClassifier started");
    info!("Using config: {:?}", config_path);
    println!("🚀 MediaClassifier - Organizing your media files...\n");
    println!("📋 Config: {}\n", config_path.display());

    // 获取目标目录
    let target_dir = if args.dir.is_empty() || args.dir == "." {
        std::env::current_dir().context("Failed to get current directory")?
    } else {
        PathBuf::from(&args.dir)
    };

    info!("Working directory: {:?}", target_dir);
    println!("📁 Working directory: {}\n", target_dir.display());

    // 扫描并收集所有媒体文件
    println!("🔍 Scanning for media files...");
    let media_files = scan_media_files(&target_dir, &config)?;

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

        match classify_file_with_config(&config, &target_dir, file) {
            Ok(result) => {
                match &result {
                    ClassifyResult::Success { from, to } => {
                        println!(
                            "{} ✅ Moved: {} → {}",
                            progress,
                            from.file_name().unwrap().to_string_lossy(),
                            to.strip_prefix(&target_dir).unwrap_or(to).display()
                        );
                    },
                    ClassifyResult::Renamed { from, to, .. } => {
                        println!(
                            "{} 🔄 Renamed: {} → {}",
                            progress,
                            from.file_name().unwrap().to_string_lossy(),
                            to.strip_prefix(&target_dir).unwrap_or(to).display()
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
    // 使用配置或命令行参数决定是否清理空目录
    let should_clean = args.clean.unwrap_or(config.global.clean_empty_dirs);
    if should_clean {
        println!("\n🧹 Cleaning up empty directories...\n");
        remove_empty_dirs(&target_dir)?;
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
        simplelog::Config::default(),
        File::create("classifier.log").context("Failed to create log file")?,
    )])
    .context("Failed to initialize logger")?;

    Ok(())
}

/// 扫描目录中的所有媒体文件
fn scan_media_files(dir: &PathBuf, config: &Config) -> Result<Vec<PathBuf>> {
    let mut media_files = Vec::new();
    let filter = FileFilter::new(&config.exclude);

    for entry in WalkDir::new(dir)
        .min_depth(1) // 跳过根目录本身
        .max_depth(9) // 限制递归深度，避免扫描太深
        .into_iter()
        .filter_entry(|e| !filter.should_exclude_entry(e) && !is_media_name_dir(e))
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

/// 检查是否为应该跳过的目录
fn is_media_name_dir(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    let name = entry.file_name().to_string_lossy();

    let low_name = name.to_lowercase();

    is_media_extension(&low_name)
}
