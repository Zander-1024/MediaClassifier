# Windows 打包说明

## 跨平台编译（在 WSL2/Linux 上编译 Windows 程序）

### 1. 安装交叉编译工具链

```bash
# 添加 Windows 目标
rustup target add x86_64-pc-windows-gnu

# 安装 MinGW 工具链
sudo apt install mingw-w64
```

### 2. 编译 Windows 版本

```bash
# 编译
cargo build --release --bin MediaClassifierGUI --target x86_64-pc-windows-gnu

# 生成的可执行文件位于:
# target/x86_64-pc-windows-gnu/release/MediaClassifierGUI.exe
```

### 3. 图标已自动嵌入

编译过程会自动将 `assets/icon.ico` 嵌入到 `.exe` 文件中。

### 4. 打包发布

创建发布包:

```bash
mkdir -p release/windows
cp target/x86_64-pc-windows-gnu/release/MediaClassifierGUI.exe release/windows/
cp default_cfg.yaml release/windows/
cp README.md release/windows/
cp LICENSE release/windows/

# 创建 ZIP 包
cd release
zip -r MediaClassifier-windows-x64.zip windows/
```

## 原生 Windows 编译

在 Windows 上直接编译:

```powershell
# 使用 MSVC 工具链
cargo build --release --bin MediaClassifierGUI

# 或使用 GNU 工具链
rustup target add x86_64-pc-windows-gnu
cargo build --release --bin MediaClassifierGUI --target x86_64-pc-windows-gnu
```

## 创建安装程序（可选）

可以使用以下工具创建 Windows 安装程序:

1. **Inno Setup** - 免费的安装程序制作工具
2. **WiX Toolset** - 创建 MSI 安装包
3. **NSIS** - 轻量级安装程序制作工具

### 使用 Inno Setup 示例

创建 `installer.iss` 文件:

```iss
[Setup]
AppName=MediaClassifier
AppVersion=1.3.0
DefaultDirName={pf}\MediaClassifier
DefaultGroupName=MediaClassifier
OutputDir=release
OutputBaseFilename=MediaClassifier-Setup
SetupIconFile=assets\icon.ico
UninstallDisplayIcon={app}\MediaClassifierGUI.exe

[Files]
Source: "target\x86_64-pc-windows-gnu\release\MediaClassifierGUI.exe"; DestDir: "{app}"
Source: "default_cfg.yaml"; DestDir: "{app}"
Source: "README.md"; DestDir: "{app}"
Source: "LICENSE"; DestDir: "{app}"

[Icons]
Name: "{group}\MediaClassifier"; Filename: "{app}\MediaClassifierGUI.exe"
Name: "{commondesktop}\MediaClassifier"; Filename: "{app}\MediaClassifierGUI.exe"
```

然后使用 Inno Setup 编译该脚本生成安装程序。

## 注意事项

1. **依赖项**: 确保目标系统有必要的运行时库（通常 Rust 编译的程序是静态链接的，无需额外依赖）
2. **字体**: WSL2 编译时需要安装中文字体，Windows 原生运行时会自动使用系统字体
3. **测试**: 建议在真实 Windows 环境中测试编译后的程序
