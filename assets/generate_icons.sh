#!/bin/bash
# 生成各种格式的应用图标
# 需要安装 inkscape 和 imagemagick

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "生成应用图标..."

# 检查依赖
if ! command -v inkscape &> /dev/null; then
    echo "错误: 需要安装 inkscape"
    echo "Ubuntu/Debian: sudo apt install inkscape"
    echo "Arch: sudo pacman -S inkscape"
    exit 1
fi

if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
    echo "错误: 需要安装 imagemagick"
    echo "Ubuntu/Debian: sudo apt install imagemagick"
    echo "Arch: sudo pacman -S imagemagick"
    exit 1
fi

# 生成不同尺寸的 PNG
echo "生成 PNG 图标..."
for size in 16 32 48 64 128 256; do
    inkscape icon.svg -o "icon-${size}.png" -w $size -h $size
    echo "  ✓ icon-${size}.png"
done

# 生成 Windows ICO 文件
echo "生成 Windows ICO 文件..."
if command -v magick &> /dev/null; then
    magick convert icon-16.png icon-32.png icon-48.png icon-64.png icon-128.png icon-256.png icon.ico
else
    convert icon-16.png icon-32.png icon-48.png icon-64.png icon-128.png icon-256.png icon.ico
fi
echo "  ✓ icon.ico"

# 生成 macOS ICNS 文件 (如果在 macOS 上)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "生成 macOS ICNS 文件..."
    mkdir -p icon.iconset
    cp icon-16.png icon.iconset/icon_16x16.png
    cp icon-32.png icon.iconset/icon_16x16@2x.png
    cp icon-32.png icon.iconset/icon_32x32.png
    cp icon-64.png icon.iconset/icon_32x32@2x.png
    cp icon-128.png icon.iconset/icon_128x128.png
    cp icon-256.png icon.iconset/icon_128x128@2x.png
    cp icon-256.png icon.iconset/icon_256x256.png
    iconutil -c icns icon.iconset
    rm -rf icon.iconset
    echo "  ✓ icon.icns"
fi

echo ""
echo "✅ 图标生成完成！"
echo ""
echo "生成的文件:"
echo "  - icon-*.png (各种尺寸的 PNG)"
echo "  - icon.ico (Windows 图标)"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  - icon.icns (macOS 图标)"
fi
