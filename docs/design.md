# MediaClassifier 设计文档

## 1. 项目概述

MediaClassifier 是一个智能媒体文件分类工具，能够自动将散乱的媒体文件按照类型和日期组织到结构化的目录中。

### 1.1 核心目标

- 自动识别媒体文件（图片、视频、音频）
- 提取文件的拍摄/创建日期
- 按 `文件类型/YYYYMMDD/文件名` 的格式组织文件
- 安全处理文件冲突
- 详细记录所有操作

### 1.2 设计原则

- **安全第一**：移动前检查冲突，避免数据丢失
- **智能识别**：优先使用 EXIF 元数据，回退到文件系统时间
- **容错处理**：遇到错误跳过并记录，不中断整体流程
- **可追溯性**：所有操作记录到日志文件

## 2. 系统架构

### 2.1 整体流程

```
┌─────────────────┐
│  开始扫描目录    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  过滤媒体文件    │ ← 根据扩展名判断
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  提取元数据      │ ← EXIF 或文件时间
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  解析日期        │ ← 转换为 YYYYMMDD
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  构建目标路径    │ ← 类型/日期/文件名
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  检查文件冲突    │ ← 比较文件大小
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
 相同大小   不同大小
    │         │
 跳过文件   重命名
    │         │
    └────┬────┘
         │
         ▼
┌─────────────────┐
│  移动文件        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  记录日志        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  处理下一个文件  │
└─────────────────┘
```

### 2.2 模块划分

```
MediaClassifier/
├── src/
│   ├── main.rs              # 主程序入口，初始化和流程控制
│   ├── media_types.rs       # 媒体类型定义和扩展名映射
│   ├── metadata.rs          # 元数据提取（EXIF、文件时间）
│   ├── classifier.rs        # 文件分类和移动逻辑
│   └── conflict.rs          # 文件冲突检测和处理
├── docs/
│   └── design.md            # 本设计文档
├── Cargo.toml               # 项目配置和依赖
└── README.md                # 用户文档
```

## 3. 模块详细设计

### 3.1 media_types.rs - 媒体类型模块

**职责**：定义支持的媒体类型和文件扩展名映射

**数据结构**：
```rust
pub enum MediaType {
    Image,
    Video,
    Audio,
}

pub struct MediaInfo {
    pub media_type: MediaType,
    pub extension: String,  // 大写，如 "JPG"
}
```

**核心函数**：
```rust
// 根据文件路径判断是否为媒体文件
pub fn get_media_info(path: &Path) -> Option<MediaInfo>

// 检查是否为图片文件（需要 EXIF 提取）
pub fn is_image_file(extension: &str) -> bool

// 获取所有支持的扩展名列表
pub fn supported_extensions() -> Vec<&'static str>
```

**支持的格式**：
- **图片**：jpg, jpeg, png, gif, tiff, tif, bmp, webp, heic, heif, nef, cr2, cr3, arw, dng, orf, raf, rw2, pef
- **视频**：mp4, mov, avi, mkv, m4v, wmv, flv, webm, mpg, mpeg, 3gp, mts, m2ts
- **音频**：mp3, flac, wav, aiff, alac, ape, aac, m4a, ogg, opus, wma

### 3.2 metadata.rs - 元数据提取模块

**职责**：从文件中提取日期信息

**核心函数**：
```rust
// 提取文件的日期（EXIF 优先，回退到文件时间）
pub fn extract_date(path: &Path, is_image: bool) -> Result<DateTime<Local>>

// 从 EXIF 数据中提取日期
fn extract_exif_date(path: &Path) -> Result<DateTime<Local>>

// 从文件系统元数据中提取日期
fn extract_file_date(path: &Path) -> Result<DateTime<Local>>
```

**EXIF 提取逻辑**：
1. 尝试读取 `DateTimeOriginal`（拍摄时间）
2. 如果失败，尝试读取 `DateTime`（修改时间）
3. 如果都失败，回退到文件系统时间

**日期解析**：
- EXIF 格式：`"2025:11:18 14:30:45"`
- 转换为：`DateTime<Local>`
- 格式化为：`"20251118"`

### 3.3 classifier.rs - 文件分类模块

**职责**：构建目标路径、创建目录、移动文件

**核心函数**：
```rust
// 分类单个文件
pub fn classify_file(source: &Path) -> Result<ClassifyResult>

// 构建目标路径：类型/YYYYMMDD/文件名
fn build_target_path(source: &Path, media_info: &MediaInfo, date: &DateTime<Local>) -> PathBuf

// 创建目标目录
fn ensure_directory(path: &Path) -> Result<()>

// 移动文件到目标位置
fn move_file(source: &Path, target: &Path) -> Result<()>
```

**目标路径格式**：
```
{扩展名大写}/{YYYYMMDD}/{原文件名}

示例：
IMG_1234.jpg → JPG/20251118/IMG_1234.jpg
DSC_5678.NEF → NEF/20251115/DSC_5678.NEF
video.mov    → MOV/20251110/video.mov
```

**返回结果**：
```rust
pub enum ClassifyResult {
    Success { from: PathBuf, to: PathBuf },
    Skipped { path: PathBuf, reason: String },
    Failed { path: PathBuf, error: String },
}
```

### 3.4 conflict.rs - 冲突处理模块

**职责**：检测文件冲突并处理

**核心函数**：
```rust
// 解决文件冲突，返回最终的目标路径
pub fn resolve_conflict(source: &Path, target: &Path) -> Result<ConflictResolution>

// 比较两个文件的大小
fn compare_file_size(path1: &Path, path2: &Path) -> Result<bool>

// 生成带后缀的新文件名
fn generate_unique_name(target: &Path) -> PathBuf
```

**冲突处理逻辑**：
```rust
pub enum ConflictResolution {
    NoConflict(PathBuf),           // 目标不存在，直接使用
    Skip(String),                   // 文件相同，跳过
    Rename(PathBuf),                // 文件不同，重命名
}
```

**重命名策略**：
```
原始：photo.jpg
存在：photo.jpg (大小不同)
结果：photo_1.jpg

如果 photo_1.jpg 也存在且不同：
结果：photo_2.jpg

依此类推...
```

### 3.5 main.rs - 主程序模块

**职责**：程序入口、日志初始化、流程控制

**主要流程**：
```rust
fn main() -> Result<()> {
    // 1. 初始化日志系统
    init_logger()?;
    
    // 2. 获取当前目录
    let current_dir = env::current_dir()?;
    
    // 3. 扫描所有文件
    let files = scan_directory(&current_dir)?;
    
    // 4. 过滤媒体文件
    let media_files = filter_media_files(files);
    
    // 5. 处理每个文件
    let mut stats = Statistics::new();
    for file in media_files {
        match classify_file(&file) {
            Ok(result) => stats.record(result),
            Err(e) => stats.record_error(&file, e),
        }
    }
    
    // 6. 输出统计信息
    stats.print_summary();
    
    Ok(())
}
```

**日志配置**：
```rust
fn init_logger() -> Result<()> {
    use simplelog::*;
    
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("classifier.log")?,
        ),
    ])?;
    
    Ok(())
}
```

**统计信息**：
```rust
struct Statistics {
    success: usize,
    skipped: usize,
    renamed: usize,
    failed: usize,
}
```

## 4. 技术选型

### 4.1 核心依赖

| 依赖库 | 版本 | 用途 |
|--------|------|------|
| `walkdir` | 2.x | 递归遍历目录树 |
| `kamadak-exif` | 0.5.x | 提取图片 EXIF 元数据 |
| `chrono` | 0.4.x | 日期时间处理和格式化 |
| `log` | 0.4.x | 日志门面（facade） |
| `simplelog` | 0.12.x | 简单的日志实现，支持文件输出 |
| `anyhow` | 1.0.x | 简化错误处理 |

### 4.2 标准库使用

- `std::fs`：文件系统操作（rename, metadata, create_dir_all）
- `std::path`：路径处理（Path, PathBuf）
- `std::env`：获取当前目录
- `std::io`：I/O 操作

## 5. 错误处理策略

### 5.1 错误分类

1. **可恢复错误**（记录日志并继续）：
   - 无法读取 EXIF 数据 → 使用文件时间
   - 文件权限不足 → 跳过该文件
   - 单个文件移动失败 → 继续处理其他文件

2. **不可恢复错误**（终止程序）：
   - 无法初始化日志系统
   - 无法获取当前目录
   - 无法创建目标目录（磁盘满、权限问题）

### 5.2 日志级别

- **INFO**：成功操作（文件移动、跳过）
- **WARN**：可恢复的问题（EXIF 读取失败、文件冲突）
- **ERROR**：操作失败（移动失败、权限错误）

### 5.3 日志格式示例

```
[2025-11-18 14:30:45] INFO  - Successfully moved: IMG_1234.jpg → JPG/20251118/IMG_1234.jpg
[2025-11-18 14:30:46] WARN  - No EXIF data found for DSC_5678.NEF, using file time
[2025-11-18 14:30:47] INFO  - Skipped (same size): video.mov → MOV/20251110/video.mov
[2025-11-18 14:30:48] WARN  - File conflict, renamed: photo.jpg → JPG/20251118/photo_1.jpg
[2025-11-18 14:30:49] ERROR - Failed to move song.mp3: Permission denied
```

## 6. 性能考虑

### 6.1 优化策略

1. **单线程处理**：文件 I/O 操作通常受限于磁盘速度，多线程收益有限
2. **延迟创建目录**：只在需要时创建目录，避免创建空目录
3. **批量操作**：使用 `walkdir` 一次性收集所有文件，避免重复遍历
4. **避免重复读取**：文件大小比较时缓存 metadata

### 6.2 内存使用

- 文件列表：O(n)，n 为文件数量
- EXIF 数据：按需读取，不常驻内存
- 日志缓冲：由 `simplelog` 管理

## 7. 安全性考虑

### 7.1 数据安全

- ✅ 移动前检查目标是否存在
- ✅ 文件大小比较确保不覆盖不同文件
- ✅ 使用 `fs::rename` 原子操作（同文件系统内）
- ✅ 所有操作记录日志，可追溯

### 7.2 边界情况

1. **跨文件系统移动**：`fs::rename` 会失败，需要 copy + delete
2. **文件名特殊字符**：保持原文件名，由操作系统处理
3. **超长路径**：依赖操作系统限制
4. **符号链接**：跟随链接处理实际文件

## 8. 测试策略

### 8.1 单元测试

- `media_types`：扩展名识别测试
- `metadata`：日期提取测试（模拟 EXIF 数据）
- `conflict`：文件名生成测试
- `classifier`：路径构建测试

### 8.2 集成测试

- 创建测试目录结构
- 准备各种类型的测试文件
- 运行分类程序
- 验证结果目录结构
- 检查日志内容

### 8.3 边界测试

- 空目录
- 只有非媒体文件
- 大量文件冲突
- 无 EXIF 数据的图片
- 权限受限的文件

## 9. 未来扩展

### 9.1 可能的功能

- [ ] 命令行参数支持（指定源目录、目标目录）
- [ ] `--dry-run` 模式（预览操作不实际执行）
- [ ] 自定义日期格式（如 `YYYY/MM/DD`）
- [ ] 支持复制模式（保留原文件）
- [ ] 进度条显示
- [ ] 并行处理（多线程）
- [ ] 配置文件支持（自定义扩展名映射）
- [ ] 撤销功能（记录操作历史）

### 9.2 技术债务

- 跨文件系统移动的处理
- 更完善的 EXIF 日期格式解析
- 视频文件元数据提取（MP4/MOV）
- 国际化支持

## 10. 总结

MediaClassifier 采用模块化设计，职责清晰，易于维护和扩展。通过智能的元数据提取和冲突处理机制，确保文件分类的准确性和安全性。详细的日志记录为用户提供了完整的操作追溯能力。

核心设计原则：
- **安全优先**：不丢失、不覆盖
- **智能识别**：EXIF 优先，文件时间兜底
- **容错处理**：单个失败不影响整体
- **可追溯性**：完整的日志记录
