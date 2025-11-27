# 应用图标和资源

## 文件说明

- `icon.svg` - 矢量图标源文件 (256x256)
- `icon.ico` - Windows 应用图标（多尺寸）
- `icon-*.png` - 各种尺寸的 PNG 图标（16, 32, 48, 64, 128, 256）
- `alipay.jpg` - 支付宝收款码
- `MediaClassifier.desktop` - Linux 桌面快捷方式模板
- `generate_icons.py` - 图标生成脚本（Python）
- `generate_icons.sh` - 图标生成脚本（需要 Inkscape）
- `install-linux.sh` - Linux 安装脚本

## 快速使用

### 生成图标（如果需要重新生成）

使用 Python 脚本（推荐，只需要 Pillow）:
```bash
cd assets
python3 generate_icons.py
```

使用 Shell 脚本（需要 Inkscape 和 ImageMagick）:
```bash
cd assets
./generate_icons.sh
```

### Linux 安装

编译后运行安装脚本：
```bash
cargo build --release --bin MediaClassifierGUI
cd assets
chmod +x install-linux.sh
./install-linux.sh
```

这将:
- 复制可执行文件到 `~/.local/bin/`
- 安装图标到 `~/.local/share/icons/`
- 创建桌面快捷方式
- 更新图标缓存

### Windows 打包

图标已经集成到构建过程中，编译时会自动嵌入：
```bash
cargo build --release --bin MediaClassifierGUI
```

生成的 `.exe` 文件会包含应用图标。

### macOS 打包

1. 使用 `generate_icons.sh` 生成 `icon.icns`（需要在 macOS 上运行）
2. 创建应用包并配置图标路径

## 图标设计说明

图标采用电影胶片和文件夹的组合设计，象征媒体文件分类：
- 背景：紫色渐变圆形
- 主体：电影胶片框架（白色）
- 元素：
  - 左右两侧的胶片孔（紫色）
  - 中间的文件夹图标（黄色）
  - 向下的箭头表示分类整理

## 生成不同格式

### Linux 使用 Inkscape:
```bash
# 生成 PNG (256x256)
inkscape icon.svg -o icon-256.png -w 256 -h 256

# 生成不同尺寸
inkscape icon.svg -o icon-128.png -w 128 -h 128
inkscape icon.svg -o icon-64.png -w 64 -h 64
inkscape icon.svg -o icon-48.png -w 48 -h 48
inkscape icon.svg -o icon-32.png -w 32 -h 32
inkscape icon.svg -o icon-16.png -w 16 -h 16
```

### Windows 使用 ImageMagick:
```bash
# 生成 ICO 文件
magick convert icon.svg -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

## 使用说明

### Windows 应用图标:
1. 将生成的 `icon.ico` 放在项目根目录
2. 在 `Cargo.toml` 中添加 `winres` build 依赖
3. 创建 `build.rs` 配置图标

### macOS 应用图标:
1. 创建 `icon.icns` 文件
2. 在应用打包时配置图标路径

### Linux 应用图标:
1. 将 PNG 文件复制到 `~/.local/share/icons/` 或系统图标目录
2. 在 `.desktop` 文件中引用图标
