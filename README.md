# MediaClassifier

一个用 Rust 编写的智能媒体文件分类工具，能够自动根据文件类型和日期将媒体文件组织到结构化的目录中。

## 功能特性

- 🎯 **智能日期提取**：图片文件优先使用 EXIF 拍摄日期，其他文件使用创建时间
- 📁 **自动分类**：按文件类型和日期创建分层目录结构（如 `JPG/20251118/photo.jpg`）
- 🔄 **安全移动**：使用文件移动操作（非复制），高效且节省空间
- 🛡️ **智能去重**：检测同名文件，相同大小则跳过，不同大小则重命名
- 📝 **详细日志**：所有操作记录到 `classifier.log` 文件，便于审计和排查
- ⚡ **仅处理媒体文件**：自动识别并只处理图片、视频和音频文件

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

在需要整理的目录中运行：

```bash
cd /path/to/your/media/files
/path/to/MediaClassifier/target/release/MediaClassifier
```

或者将编译后的可执行文件复制到目标目录：

```bash
cp target/release/MediaClassifier /path/to/your/media/files/
cd /path/to/your/media/files
./MediaClassifier
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

## 工作原理

1. **扫描文件**：递归遍历当前目录下的所有文件
2. **识别媒体**：根据文件扩展名判断是否为媒体文件
3. **提取日期**：
   - 图片文件：尝试读取 EXIF 中的 `DateTimeOriginal` 或 `DateTime` 字段
   - 视频/音频：使用文件的创建时间（或修改时间）
4. **创建目录**：按 `文件类型/YYYYMMDD/` 格式创建目录
5. **处理冲突**：
   - 如果目标文件已存在，比较文件大小
   - 大小相同：跳过移动，记录日志
   - 大小不同：在文件名后添加数字后缀（如 `photo_1.jpg`）
6. **移动文件**：将文件移动到目标目录
7. **记录日志**：所有操作写入日志文件

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

## 技术栈

- **Rust 2024 Edition**
- **walkdir**：高效的目录遍历
- **kamadak-exif**：EXIF 元数据提取
- **chrono**：日期时间处理
- **log + simplelog**：日志记录
- **anyhow**：错误处理

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

## 更新日志

### v0.1.0 (2025-11-18)
- 初始版本
- 支持图片、视频、音频文件分类
- EXIF 日期提取
- 智能文件冲突处理
- 日志记录功能
