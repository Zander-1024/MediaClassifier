# MediaClassifier 使用示例

## 基本使用场景

### 场景 1：整理相机导出的照片

假设你从相机导出了一批照片到 `~/Photos/import` 目录：

```bash
cd ~/Photos/import
ls
# IMG_1234.JPG  IMG_1235.JPG  DSC_5678.NEF  DSC_5679.NEF  video.MOV
```

运行 MediaClassifier：

```bash
/path/to/MediaClassifier
```

结果：

```
~/Photos/import/
├── JPG/
│   └── 20251118/
│       ├── IMG_1234.JPG
│       └── IMG_1235.JPG
├── NEF/
│   └── 20251115/
│       ├── DSC_5678.NEF
│       └── DSC_5679.NEF
└── MOV/
    └── 20251110/
        └── video.MOV
```

### 场景 2：处理混合媒体文件

目录中有各种类型的文件：

```bash
cd ~/Downloads
ls
# photo.jpg  song.mp3  video.mp4  document.pdf  archive.zip
```

运行后：

```
~/Downloads/
├── JPG/
│   └── 20251118/
│       └── photo.jpg
├── MP3/
│   └── 20251101/
│       └── song.mp3
├── MP4/
│   └── 20251115/
│       └── video.mp4
├── document.pdf      # 非媒体文件保持不变
└── archive.zip       # 非媒体文件保持不变
```

### 场景 3：处理文件冲突

#### 情况 A：文件完全相同（跳过）

```bash
# 已存在：JPG/20251118/photo.jpg (2.5 MB)
# 新文件：photo.jpg (2.5 MB，内容相同)

# 运行结果：
[1/1] ⏭️  Skipped: photo.jpg (already exists)

# 文件保持原样，不移动
```

#### 情况 B：文件名相同但内容不同（重命名）

```bash
# 已存在：JPG/20251118/photo.jpg (2.5 MB)
# 新文件：photo.jpg (3.1 MB，内容不同)

# 运行结果：
[1/1] 🔄 Renamed: photo.jpg → JPG/20251118/photo_1.jpg

# 最终结构：
JPG/20251118/
├── photo.jpg      # 原有文件
└── photo_1.jpg    # 新文件（重命名）
```

### 场景 4：批量整理多个子目录

```bash
cd ~/Media
tree
# .
# ├── 2024-vacation/
# │   ├── IMG_001.jpg
# │   └── IMG_002.jpg
# ├── old-photos/
# │   ├── photo1.jpg
# │   └── photo2.nef
# └── random/
#     ├── video.mov
#     └── song.mp3

/path/to/MediaClassifier

# 结果：所有媒体文件按类型和日期重新组织
# .
# ├── JPG/
# │   ├── 20240801/
# │   │   ├── IMG_001.jpg
# │   │   └── IMG_002.jpg
# │   └── 20200315/
# │       └── photo1.jpg
# ├── NEF/
# │   └── 20200315/
# │       └── photo2.nef
# ├── MOV/
# │   └── 20240805/
# │       └── video.mov
# ├── MP3/
# │   └── 20230610/
# │       └── song.mp3
# ├── 2024-vacation/    # 空目录可手动删除
# ├── old-photos/       # 空目录可手动删除
# └── random/           # 空目录可手动删除
```

## 日志文件示例

### 成功处理的日志

```
14:30:45 [INFO] MediaClassifier started
14:30:45 [INFO] Working directory: "/home/user/Photos"
14:30:45 [INFO] Found 10 media files
14:30:45 [INFO] Successfully moved: "IMG_1234.jpg" → "JPG/20251118/IMG_1234.jpg"
14:30:45 [INFO] Successfully moved: "DSC_5678.NEF" → "NEF/20251115/DSC_5678.NEF"
14:30:45 [INFO] Classification completed: 10 success, 0 renamed, 0 skipped, 0 failed
```

### 包含警告的日志

```
14:30:45 [WARN] Failed to extract EXIF date for "photo.jpg": Failed to read EXIF data, falling back to file time
14:30:45 [INFO] Successfully moved: "photo.jpg" → "JPG/20251118/photo.jpg"
```

### 包含冲突处理的日志

```
14:30:45 [INFO] Skipped: "photo.jpg" - File already exists with same size (2500000 bytes)
14:30:46 [WARN] File renamed due to conflict: "photo.jpg" → "JPG/20251118/photo_1.jpg"
```

### 包含错误的日志

```
14:30:45 [ERROR] Failed to move "locked.jpg": Permission denied
```

## 常见问题

### Q: 如何只整理特定类型的文件？

A: 目前程序会处理所有支持的媒体文件。如果只想处理特定类型，可以先将其他类型的文件移到别处。

### Q: 可以撤销操作吗？

A: 程序使用移动操作，文件仍在同一磁盘上。可以查看 `classifier.log` 了解所有操作，手动移回原位置。

### Q: 如何处理没有 EXIF 数据的图片？

A: 程序会自动回退到使用文件的创建时间或修改时间。

### Q: 程序会删除空目录吗？

A: 不会。原始的空目录会保留，需要手动删除。

### Q: 如何在不同目录之间移动文件？

A: 将可执行文件复制到目标目录，或在目标目录中运行：

```bash
cd /path/to/target/directory
/path/to/MediaClassifier
```

### Q: 支持的最大文件数量是多少？

A: 理论上没有限制，但建议分批处理大量文件（如超过 10000 个）。

## 高级技巧

### 技巧 1：预览操作（不实际移动）

虽然程序没有内置 dry-run 模式，但可以先在测试目录中运行：

```bash
# 创建测试目录
mkdir test_run
cp -r /path/to/media/* test_run/
cd test_run

# 运行分类
/path/to/MediaClassifier

# 检查结果
ls -R

# 如果满意，在原目录运行
cd /path/to/media
/path/to/MediaClassifier
```

### 技巧 2：批量处理多个目录

创建一个脚本：

```bash
#!/bin/bash
CLASSIFIER="/path/to/MediaClassifier"

for dir in ~/Photos/*/; do
    echo "Processing $dir"
    cd "$dir"
    $CLASSIFIER
done
```

### 技巧 3：定期自动整理

添加到 crontab（每天凌晨 2 点运行）：

```bash
0 2 * * * cd ~/Downloads && /path/to/MediaClassifier >> ~/classifier_cron.log 2>&1
```

### 技巧 4：整理后清理空目录

```bash
# 运行分类器
/path/to/MediaClassifier

# 删除空目录
find . -type d -empty -delete
```

## 性能参考

在标准硬盘上的处理速度参考：

- **小文件（< 1MB）**：约 1000 文件/秒
- **中等文件（1-10MB）**：约 500 文件/秒
- **大文件（> 10MB）**：受限于磁盘 I/O

实际速度取决于：
- 磁盘类型（SSD vs HDD）
- 文件大小
- EXIF 数据复杂度
- 文件系统类型

## 故障排除

### 问题：Permission denied

**原因**：没有文件或目录的访问权限

**解决**：
```bash
# 检查权限
ls -l problematic_file.jpg

# 修改权限（如果是你的文件）
chmod 644 problematic_file.jpg
```

### 问题：No space left on device

**原因**：磁盘空间不足

**解决**：
```bash
# 检查磁盘空间
df -h

# 清理空间或移动到其他磁盘
```

### 问题：程序运行很慢

**可能原因**：
1. 文件数量太多
2. 网络驱动器或慢速磁盘
3. 大量 EXIF 数据读取

**解决**：
- 分批处理文件
- 使用本地磁盘
- 检查日志中的警告信息

## 总结

MediaClassifier 是一个简单但强大的媒体文件整理工具。通过智能的日期提取和冲突处理，可以快速将混乱的媒体文件组织成清晰的目录结构。配合日志文件，所有操作都可追溯和审计。
