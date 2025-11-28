mod config_display;

use anyhow::{Context, Result};
use clap::Parser;
use config_display::show_config;
use log::info;
use mc_lib::{
    ClassifyResult, Config, FileFilter, classify_file_with_config, get_media_info,
    remove_empty_dirs,
};
use simplelog::*;
use std::fs::File;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

    fn record(&mut self, result: &ClassifyResult) {
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

/// 获取日志文件的绝对路径
fn get_log_file_path(target_dir: &Path) -> PathBuf {
    target_dir.join("classifier.log")
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

    // 获取目标目录
    let target_dir = if args.dir.is_empty() || args.dir == "." {
        std::env::current_dir().context("Failed to get current directory")?
    } else {
        PathBuf::from(&args.dir)
    };

    // 初始化日志系统
    let log_path = get_log_file_path(&target_dir);
    init_logger(&log_path)?;

    info!("MediaClassifier started");
    info!("Using config: {:?}", config_path);
    println!("🚀 MediaClassifier - Organizing your media files...\n");
    println!("📋 Config: {}\n", config_path.display());

    info!("Working directory: {:?}", target_dir);
    println!("📁 Working directory: {}\n", target_dir.display());

    // 扫描并收集所有媒体文件
    print!("🔍 Scanning for media files...");
    stdout().flush().ok();
    let (media_files, skipped_dirs) = scan_media_files(&target_dir, &config)?;

    if media_files.is_empty() {
        println!(" Done");
        println!("ℹ️  No media files found in the current directory.");
        info!("No media files found");
        return Ok(());
    }

    println!(" Found {} files", media_files.len());
    info!("Found {} media files", media_files.len());

    // 记录跳过的目录到日志
    if !skipped_dirs.is_empty() {
        info!("Skipped directories:");
        for dir in &skipped_dirs {
            info!("  [SKIP DIR] {}", dir.display());
        }
    }

    // 处理每个文件
    println!("⚙️  Processing files...");
    let mut stats = Statistics::new();
    let total = media_files.len();

    for (index, file) in media_files.iter().enumerate() {
        // 在终端显示进度（覆盖同一行）
        print!("\r⚙️  Processing: [{}/{}]", index + 1, total);
        let _ = stdout().flush();

        match classify_file_with_config(&config, &target_dir, file) {
            Ok(result) => {
                // 记录详细日志到文件
                log_result(&result);
                stats.record(&result);
            },
            Err(e) => {
                info!("[ERROR] {} -> {}", file.display(), e);
                stats.failed += 1;
            },
        }
    }

    // 清除进度行并打印完成信息
    print!("\r⚙️  Processing: [{}/{}] ✓\n", total, total);

    // 使用配置或命令行参数决定是否清理空目录
    let should_clean = args.clean.unwrap_or(config.global.clean_empty_dirs);
    if should_clean {
        println!("🧹 Cleaning up empty directories...");
        remove_empty_dirs(&target_dir)?;
    }

    // 打印统计信息
    stats.print_summary();

    // 显示日志文件路径
    println!("📝 Detailed logs saved to: {}", log_path.display());
    println!("✨ Done!\n");

    Ok(())
}

/// 记录分类结果到日志文件
fn log_result(result: &ClassifyResult) {
    match result {
        ClassifyResult::Success { from, to } => {
            info!("[SUCCESS] {} -> {}", from.display(), to.display());
        },
        ClassifyResult::Renamed { from, to } => {
            info!("[RENAMED] {} -> {}", from.display(), to.display());
        },
        ClassifyResult::Skipped { path, reason } => {
            info!("[SKIPPED] {} | Reason: {}", path.display(), reason);
        },
        ClassifyResult::Failed { path, error } => {
            info!("[FAILED] {} | Error: {}", path.display(), error);
        },
    }
}

/// 初始化日志系统
fn init_logger(log_path: &Path) -> Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        simplelog::Config::default(),
        File::create(log_path).context("Failed to create log file")?,
    )])
    .context("Failed to initialize logger")?;

    Ok(())
}

/// 扫描目录中的所有媒体文件
/// 返回 (媒体文件列表, 跳过的目录列表)
fn scan_media_files(dir: &PathBuf, config: &Config) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut media_files = Vec::new();
    let mut skipped_dirs = Vec::new();
    let filter = FileFilter::new(&config.exclude);

    // 首先收集被跳过的目录
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .max_depth(9)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_dir() && filter.should_exclude_entry(&entry) {
            skipped_dirs.push(entry.into_path());
        }
    }

    // 收集媒体文件
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .max_depth(9)
        .into_iter()
        .filter_entry(|e| !filter.should_exclude_entry(e))
    {
        let entry = entry.context("Failed to read directory entry")?;

        // 只处理文件
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // 检查是否为媒体文件
        if get_media_info(path).is_some() {
            media_files.push(path.to_path_buf());
        }
    }

    Ok((media_files, skipped_dirs))
}
