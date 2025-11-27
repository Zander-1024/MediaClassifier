#!/bin/bash
# Linux 平台安装脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "📦 安装 MediaClassifier..."

# 检查是否已编译
BINARY_PATH="$PROJECT_ROOT/target/release/MediaClassifierGUI"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ 未找到编译后的程序"
    echo "请先运行: cargo build --release --bin MediaClassifierGUI"
    exit 1
fi

# 创建必要的目录
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
mkdir -p ~/.local/share/icons/hicolor/128x128/apps
mkdir -p ~/.local/share/icons/hicolor/64x64/apps
mkdir -p ~/.local/share/icons/hicolor/48x48/apps
mkdir -p ~/.local/share/icons/hicolor/32x32/apps
mkdir -p ~/.local/share/icons/hicolor/16x16/apps

# 复制二进制文件
echo "  ✓ 安装可执行文件到 ~/.local/bin/"
cp "$BINARY_PATH" ~/.local/bin/MediaClassifierGUI
chmod +x ~/.local/bin/MediaClassifierGUI

# 复制图标
echo "  ✓ 安装图标文件"
cp "$SCRIPT_DIR/icon-256.png" ~/.local/share/icons/hicolor/256x256/apps/mediaclassifier.png
cp "$SCRIPT_DIR/icon-128.png" ~/.local/share/icons/hicolor/128x128/apps/mediaclassifier.png
cp "$SCRIPT_DIR/icon-64.png" ~/.local/share/icons/hicolor/64x64/apps/mediaclassifier.png
cp "$SCRIPT_DIR/icon-48.png" ~/.local/share/icons/hicolor/48x48/apps/mediaclassifier.png
cp "$SCRIPT_DIR/icon-32.png" ~/.local/share/icons/hicolor/32x32/apps/mediaclassifier.png
cp "$SCRIPT_DIR/icon-16.png" ~/.local/share/icons/hicolor/16x16/apps/mediaclassifier.png

# 创建 desktop 文件
echo "  ✓ 创建桌面快捷方式"
cat > ~/.local/share/applications/MediaClassifier.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=MediaClassifier
GenericName=Media File Organizer
Comment=Rule-based media file automatic classification tool
Comment[zh_CN]=基于规则的媒体文件自动分类工具
Exec=$HOME/.local/bin/MediaClassifierGUI
Icon=mediaclassifier
Terminal=false
Categories=Utility;FileTools;
Keywords=media;file;organizer;classifier;
StartupNotify=true
EOF

chmod +x ~/.local/share/applications/MediaClassifier.desktop

# 更新图标缓存
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "  ✓ 更新图标缓存"
    gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
fi

if command -v update-desktop-database &> /dev/null; then
    echo "  ✓ 更新桌面数据库"
    update-desktop-database ~/.local/share/applications 2>/dev/null || true
fi

echo ""
echo "✅ 安装完成！"
echo ""
echo "你现在可以:"
echo "  1. 从应用菜单启动 MediaClassifier"
echo "  2. 在终端运行: MediaClassifierGUI"
echo ""
echo "注意: 确保 ~/.local/bin 在你的 PATH 中"
echo "如果找不到命令，请添加以下行到 ~/.bashrc 或 ~/.zshrc:"
echo '  export PATH="$HOME/.local/bin:$PATH"'
