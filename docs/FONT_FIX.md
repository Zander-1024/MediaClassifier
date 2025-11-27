# WSL2 中文字体修复指南

## 问题描述
在 WSL2 上使用 Slint 构建 GUI 时，中文显示为方块字（口口口），这是因为系统缺少中文字体。

## 解决方案 1：安装系统中文字体（推荐）✅

### 1. 安装字体包
```bash
sudo apt update
sudo apt install -y fonts-noto-cjk fonts-wqy-zenhei fonts-wqy-microhei
```

安装的字体：
- **Noto Sans CJK**: Google 开源的高质量中日韩字体
- **文泉驿正黑 (WenQuanYi Zen Hei)**: 开源中文黑体
- **文泉驿微米黑 (WenQuanYi Micro Hei)**: 开源中文微米黑

### 2. 刷新字体缓存
```bash
fc-cache -fv
```

### 3. 验证安装
```bash
fc-list :lang=zh | head
```

应该能看到类似输出：
```
/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc: Noto Sans CJK SC:style=Regular
/usr/share/fonts/truetype/wqy/wqy-microhei.ttc: WenQuanYi Micro Hei,文泉驛微米黑:style=Regular
```

### 4. 重新编译并运行
```bash
cargo build --release --bin MediaClassifierGUI
DISPLAY=:0 ./target/release/MediaClassifierGUI
```

## 解决方案 2：在 Slint 中指定字体

如果需要更精确的字体控制，可以在 Slint 文件中指定：

```slint
Text {
    text: "中文测试";
    font-family: "Noto Sans CJK SC", "WenQuanYi Micro Hei", sans-serif;
}
```

## 常见问题

### Q1: 程序启动时有 xdg color schemes 警告
这是正常的，不影响使用。WSL2 中缺少某些桌面服务，但不影响 GUI 显示。

### Q2: 字体还是显示为方块
- 确认字体已安装：`fc-list :lang=zh`
- 重新登录 WSL2 会话
- 确保 DISPLAY 环境变量正确设置

### Q3: 如何在 Windows 上查看 WSL2 GUI？
需要安装 X Server（如 VcXsrv、Xming）或使用 WSL2 的内置 GUI 支持（Windows 11）。

## 其他可选字体

```bash
# 思源黑体
sudo apt install fonts-noto-cjk

# 文泉驿点阵宋体
sudo apt install fonts-wqy-bitmapsong

# Adobe 思源宋体
sudo apt install fonts-adobe-source-han-serif-cn
```

## 参考资料
- [Slint Documentation](https://slint.dev/)
- [FontConfig Manual](https://www.freedesktop.org/software/fontconfig/fontconfig-user.html)
- [WSL GUI Apps](https://learn.microsoft.com/en-us/windows/wsl/tutorials/gui-apps)
