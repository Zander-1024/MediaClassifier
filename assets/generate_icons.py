#!/usr/bin/env python3
"""
使用 Python PIL/Pillow 生成应用图标
不依赖 inkscape，但需要先有一个基础的 PNG 图片
"""

import os
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("错误: 需要安装 Pillow")
    print("运行: pip install Pillow")
    sys.exit(1)

SCRIPT_DIR = Path(__file__).parent
SIZES = [16, 32, 48, 64, 128, 256]

def create_simple_icon():
    """创建一个简单的应用图标（如果没有 PNG）"""
    print("创建简单图标...")
    
    # 创建 256x256 的基础图标
    from PIL import ImageDraw, ImageFont
    
    img = Image.new('RGB', (256, 256), color=(102, 126, 234))
    draw = ImageDraw.Draw(img)
    
    # 绘制白色圆角矩形
    draw.rounded_rectangle([40, 60, 216, 196], radius=16, fill='white')
    
    # 绘制简单的文件夹形状
    draw.rectangle([80, 100, 176, 160], fill='#fbbf24')
    draw.polygon([(80, 100), (110, 100), (120, 90), (150, 90), (150, 100)], fill='#fbbf24')
    
    # 绘制电影符号
    draw.ellipse([110, 120, 146, 156], fill='#667eea')
    draw.text((120, 125), "🎬", fill='white')
    
    return img

def main():
    os.chdir(SCRIPT_DIR)
    
    # 尝试查找现有的 PNG，如果没有则创建一个
    base_image_path = SCRIPT_DIR / "icon-256.png"
    
    if not base_image_path.exists():
        print("未找到 icon-256.png，创建简单图标...")
        base_img = create_simple_icon()
        base_img.save(base_image_path)
        print(f"  ✓ 已创建 {base_image_path}")
    else:
        print(f"使用现有图标: {base_image_path}")
        base_img = Image.open(base_image_path)
    
    # 生成不同尺寸
    print("\n生成不同尺寸的 PNG...")
    for size in SIZES:
        if size == 256:
            continue
        output_path = SCRIPT_DIR / f"icon-{size}.png"
        resized = base_img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(output_path)
        print(f"  ✓ icon-{size}.png")
    
    # 生成 ICO 文件
    print("\n生成 Windows ICO 文件...")
    ico_images = []
    for size in SIZES:
        img_path = SCRIPT_DIR / f"icon-{size}.png"
        if img_path.exists():
            ico_images.append(Image.open(img_path))
    
    if ico_images:
        ico_path = SCRIPT_DIR / "icon.ico"
        ico_images[0].save(
            ico_path,
            format='ICO',
            sizes=[(img.size[0], img.size[1]) for img in ico_images]
        )
        print(f"  ✓ icon.ico")
    
    print("\n✅ 图标生成完成！")
    print(f"\n生成的文件位于: {SCRIPT_DIR}")
    print("  - icon-*.png (各种尺寸)")
    print("  - icon.ico (Windows 图标)")

if __name__ == "__main__":
    main()
