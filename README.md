# MediaClassifier

一个用 Rust 编写的智能媒体文件分类工具，能够自动根据文件类型和日期将媒体文件组织到结构化的目录中。

## 功能特性

- 🎯 **智能日期提取**：图片文件优先使用 EXIF 拍摄日期，其他文件使用创建时间
- 📁 **灵活分类规则**：通过 YAML 配置文件自定义分类策略，支持按文件大小、类型、日期等多维度组织
- 🎨 **自定义目录结构**：支持模板变量（`{ext}`, `{year}`, `{month}` 等），自由定制目录层级
- 🔄 **安全移动**：使用文件移动操作（非复制），高效且节省空间
- 🛡️ **智能去重**：检测同名文件，相同大小则跳过，不同大小则重命名
- 📝 **详细日志**：所有操作记录到 `classifier.log` 文件，便于审计和排查
- ⚡ **仅处理媒体文件**：自动识别并只处理图片、视频和音频文件
- 🎯 **规则优先级**：多条规则按顺序匹配，灵活处理不同场景

## 支持的文件格式

### 图片格式
- **常规格式**：JPG, JPEG, PNG, GIF, TIFF, TIF, BMP, WEBP, HEIC, HEIF
- **RAW 格式**：NEF (Nikon), CR2/CR3 (Canon), ARW (Sony), DNG (Adobe), ORF (Olympus), RAF (Fujifilm), RW2 (Panasonic), PEF (Pentax)

### 视频格式
- MP4, MOV, AVI, MKV, M4V, WMV, FLV, WEBM, MPG, MPEG, 3GP, MTS, M2TS

### 音频格式
- **无损**：FLAC, WAV, AIFF, ALAC, APE
- **有损**：MP3, AAC, M4A, OGG, OPUS, WMA

## 安装

### 方式 1：下载预编译版本（推荐）

从 [Releases 页面](https://github.com/Zander-1024/MediaClassifier/releases) 下载适合你系统的版本：

**Linux (x86_64)**:
```bash
wget https://github.com/Zander-1024/MediaClassifier/releases/latest/download/MediaClassifier-linux-x86_64.tar.gz
tar xzf MediaClassifier-linux-x86_64.tar.gz
chmod +x MediaClassifier
sudo mv MediaClassifier /usr/local/bin/
```

**macOS (Apple Silicon)**:
```bash
wget https://github.com/Zander-1024/MediaClassifier/releases/latest/download/MediaClassifier-macos-aarch64.tar.gz
tar xzf MediaClassifier-macos-aarch64.tar.gz
chmod +x MediaClassifier
sudo mv MediaClassifier /usr/local/bin/
```

**macOS (Intel)**:
```bash
wget https://github.com/Zander-1024/MediaClassifier/releases/latest/download/MediaClassifier-macos-x86_64.tar.gz
tar xzf MediaClassifier-macos-x86_64.tar.gz
chmod +x MediaClassifier
sudo mv MediaClassifier /usr/local/bin/
```

**Windows**:
1. 下载 `MediaClassifier-windows-x86_64.exe.zip`
2. 解压到你想要的位置
3. 将解压目录添加到系统 PATH（可选）

### 方式 2：从源码编译

确保已安装 Rust 工具链（1.70+）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

克隆并编译项目：

```bash
git clone https://github.com/Zander-1024/MediaClassifier.git
cd MediaClassifier
cargo build --release

# 可执行文件位于 target/release/MediaClassifier
```

## 使用方法

### 基本用法

**使用默认配置：**
```bash
cd /path/to/your/media/files
MediaClassifier
```

**使用自定义配置文件：**
```bash
MediaClassifier -f /path/to/config.yaml
```

**查看当前配置：**
```bash
MediaClassifier -s
```

**指定处理目录：**
```bash
MediaClassifier -d /path/to/media/files
```

**禁用空目录清理：**
```bash
MediaClassifier --clean=false
```

### 命令行参数

```
Options:
  -d, --dir <DIR>              处理的目录 [默认: 当前目录]
  -f, --file <FILE>            配置文件路径 [默认: ~/.config/media-classifier/config.yaml]
  -s, --show-config            显示当前配置（表格格式）
  -c, --configure              交互式配置模式（开发中）
      --clean <CLEAN>          处理后是否清理空目录 [默认: true]
  -h, --help                   显示帮助信息
  -V, --version                显示版本信息
```

### 运行示例

假设当前目录有以下文件：
```
.
├── IMG_1234.jpg
├── DSC_5678.NEF
├── video.mov
├── song.mp3
└── document.pdf
```

运行后将变成：
```
.
├── JPG/
│   └── 20251118/
│       └── IMG_1234.jpg
├── NEF/
│   └── 20251115/
│       └── DSC_5678.NEF
├── MOV/
│   └── 20251110/
│       └── video.mov
├── MP3/
│   └── 20251101/
│       └── song.mp3
├── document.pdf          # 非媒体文件保持不变
└── classifier.log        # 操作日志
```

### 日志文件

所有操作都会记录到 `classifier.log` 文件中，包括：
- ✅ 成功移动的文件
- ⏭️ 跳过的文件（已存在且大小相同）
- 🔄 重命名的文件（已存在但大小不同）
- ❌ 失败的操作（权限问题、读取错误等）

查看日志：
```bash
cat classifier.log
# 或实时查看
tail -f classifier.log
```

## 配置文件

首次运行时，程序会在 `~/.config/media-classifier/config.yaml` 自动生成带详细注释的配置文件。

### 配置示例

```yaml
global:
  date_format: "YYYYMMDD"
  directory_template: "{ext}/{date}"
  clean_empty_dirs: true

rules:
  # 大尺寸照片
  - name: "High Quality Photos"
    extensions: [jpg, jpeg, png]
    file_size:
      min: "5MB"
      max: "0B"  # 0 表示不限制
    directory_template: "Photos/{year}/{month}"
    enabled: true

  # RAW 格式
  - name: "RAW Photos"
    extensions: [nef, cr2, cr3, arw, dng]
    directory_template: "RAW/{year}/{month}/{day}"
    enabled: true

  # 视频文件
  - name: "Videos"
    extensions: [mp4, mov, avi, mkv]
    directory_template: "Videos/{year}"
    enabled: true
```

### 模板变量

- `{type}` - 媒体类型（Image/Video/Audio）
- `{ext}` - 文件扩展名（大写）
- `{year}` - 年份（YYYY）
- `{month}` - 月份（MM）
- `{day}` - 日期（DD）
- `{date}` - 根据 date_format 格式化的日期

### 日期格式

- `YYYY` - 2025
- `YYYYMM` - 202511
- `YYYYMMDD` - 20251118
- `YYYY/MM` - 2025/11
- `YYYY/MM/DD` - 2025/11/18
- 或自定义格式

### 文件大小单位

- **Byte**: `B`, `KB`, `MB`, `GB`, `TB`
- **bit**: `b`, `Kb`, `Mb`, `Gb`, `Tb` (1 Byte = 8 bits)
- 支持小数：`1.5MB`, `500KB`
- `0B` 表示不限制

### 配置示例场景

**场景 1：按年月日分层**
```yaml
directory_template: "{ext}/{year}/{month}/{day}"
# 结果: JPG/2025/11/18/photo.jpg
```

**场景 2：按大小区分照片**
```yaml
rules:
  - name: "Large Photos"
    extensions: [jpg]
    file_size: { min: "10MB", max: "0B" }
    directory_template: "Photos/Large/{year}"
  
  - name: "Small Photos"
    extensions: [jpg]
    file_size: { min: "0B", max: "10MB" }
    directory_template: "Photos/Small/{date}"
```

**场景 3：音乐不使用日期**
```yaml
- name: "Music"
  extensions: [mp3, flac]
  directory_template: "Music/{ext}"
  date_format: null  # 不使用日期
```

## 工作原理

1. **加载配置**：读取配置文件或使用默认配置
2. **扫描文件**：递归遍历目录下的所有媒体文件
3. **规则匹配**：按顺序匹配规则（扩展名 + 文件大小）
4. **提取日期**：
   - 图片文件：尝试读取 EXIF 中的 `DateTimeOriginal` 或 `DateTime` 字段
   - 视频/音频：使用文件的创建时间（或修改时间）
5. **构建路径**：根据规则的模板和变量生成目标路径
6. **处理冲突**：
   - 如果目标文件已存在，比较文件大小
   - 大小相同：跳过移动，记录日志
   - 大小不同：在文件名后添加数字后缀（如 `photo_1.jpg`）
7. **移动文件**：将文件移动到目标目录
8. **记录日志**：所有操作写入日志文件

## 冲突处理策略

当目标位置已存在同名文件时：

```
原文件: photo.jpg (2.5 MB)
目标位置已有: JPG/20251118/photo.jpg (2.5 MB)
→ 跳过（文件相同）

原文件: photo.jpg (3.1 MB)
目标位置已有: JPG/20251118/photo.jpg (2.5 MB)
→ 重命名为 photo_1.jpg 并移动
```

## 注意事项

⚠️ **重要提示**：
- 本工具会**移动**文件（非复制），请确保在操作前备份重要数据
- 首次使用建议在测试目录中试运行
- 程序会跳过已创建的分类目录，避免重复处理
- Linux 系统上文件创建时间可能不准确，建议主要用于有 EXIF 数据的图片

## 高级用法

### 查看配置

```bash
MediaClassifier -s
```

输出示例：
```
📋 Configuration: ~/.config/media-classifier/config.yaml

Global Settings:
╭────────────────────────┬──────────────╮
│ Setting                ┆ Value        │
╞════════════════════════╪══════════════╡
│ Date Format            ┆ YYYYMMDD     │
│ Directory Template     ┆ {ext}/{date} │
│ Clean Empty Dirs       ┆ true         │
╰────────────────────────┴──────────────╯

Classification Rules (6 enabled):
╭───┬─────────────────────┬──────────────┬──────────┬──────────┬─────────────────────╮
│ # ┆ Name                ┆ Extensions   ┆ Min Size ┆ Max Size ┆ Directory Template  │
╞═══╪═════════════════════╪══════════════╪══════════╪══════════╪═════════════════════╡
│ 1 ┆ High Quality Photos ┆ jpg,jpeg,png ┆ 5.00MB   ┆ ∞        ┆ Photos/{year}/{mon} │
│ 2 ┆ RAW Photos          ┆ nef,cr2...   ┆ ∞        ┆ ∞        ┆ RAW/{year}/{month}  │
...
╰───┴─────────────────────┴──────────────┴──────────┴──────────┴─────────────────────╯
```

### 编辑配置文件

```bash
# Linux/macOS
nano ~/.config/media-classifier/config.yaml

# 或使用你喜欢的编辑器
code ~/.config/media-classifier/config.yaml
```

### 使用不同配置处理不同目录

```bash
# 处理照片
MediaClassifier -f ~/configs/photos.yaml -d ~/Photos

# 处理视频
MediaClassifier -f ~/configs/videos.yaml -d ~/Videos

# 处理音乐
MediaClassifier -f ~/configs/music.yaml -d ~/Music
```

## 技术栈

- **Rust 2024 Edition**
- **walkdir**：高效的目录遍历
- **kamadak-exif**：EXIF 元数据提取
- **chrono**：日期时间处理
- **log + simplelog**：日志记录
- **anyhow**：错误处理
- **serde + serde_yaml_bw**：配置文件解析 (使用活跃维护的 YAML 库)
- **comfy-table**：表格显示
- **clap**：命令行参数解析

## 开发

### 构建
```bash
cargo build
```

### 运行测试
```bash
cargo test
```

### 调试模式
```bash
cargo run
```

### 发布构建
```bash
cargo build --release
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 常见问题

### Q: 如何自定义分类规则？

A: 编辑配置文件 `~/.config/media-classifier/config.yaml`，添加或修改 `rules` 部分。规则按顺序匹配，第一个匹配成功的规则会被应用。

### Q: 如何只处理特定大小的文件？

A: 在规则中设置 `file_size`：
```yaml
file_size:
  min: "1MB"   # 只处理 ≥1MB 的文件
  max: "100MB" # 只处理 ≤100MB 的文件
```

### Q: 如何按年月分层而不是单层目录？

A: 修改 `directory_template`：
```yaml
directory_template: "{ext}/{year}/{month}"
# 结果: JPG/2025/11/photo.jpg
```

### Q: 配置文件在哪里？

A: 
- Linux: `~/.config/media-classifier/config.yaml`
- macOS: `~/.config/media-classifier/config.yaml`
- Windows: `C:\Users\<用户名>\.config\media-classifier\config.yaml`

### Q: 可以撤销操作吗？

A: 程序使用移动操作，文件仍在同一磁盘上。查看 `classifier.log` 了解所有操作，可以手动移回原位置。

### Q: 如何处理没有 EXIF 数据的图片？

A: 程序会自动回退到使用文件的创建时间或修改时间。

## 更新日志

### v1.2.0 (2025-11-24)
- ✨ 新增 YAML 配置文件支持
- ✨ 支持自定义分类规则和目录模板
- ✨ 支持文件大小过滤
- ✨ 支持多种日期格式
- ✨ 新增配置显示功能（`-s`）
- ✨ 支持扩展名别名
- ✨ 支持排除规则
- 🐛 修复规则匹配优先级问题
- 📝 完善文档和示例

### v0.1.0 (2025-11-18)
- 初始版本
- 支持图片、视频、音频文件分类
- EXIF 日期提取
- 智能文件冲突处理
- 日志记录功能
