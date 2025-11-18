# 发布指南

## 如何创建新版本

### 1. 更新版本号

编辑 `Cargo.toml`，更新版本号：

```toml
[package]
version = "0.2.0"  # 更新这里
```

### 2. 更新 CHANGELOG

在 `README.md` 或创建 `CHANGELOG.md` 中记录更新内容：

```markdown
## v0.2.0 (2025-11-20)
- 新增功能 A
- 修复 Bug B
- 性能优化 C
```

### 3. 提交更改

```bash
git add Cargo.toml README.md
git commit -m "Bump version to 0.2.0"
git push origin main
```

### 4. 创建并推送标签

```bash
# 创建标签
git tag -a v0.2.0 -m "Release version 0.2.0"

# 推送标签到 GitHub
git push origin v0.2.0
```

### 5. 自动构建

推送标签后，GitHub Actions 会自动：
1. 编译所有平台的二进制文件
2. 创建 Release
3. 上传编译好的文件到 Release

### 6. 检查 Release

访问 `https://github.com/你的用户名/MediaClassifier/releases` 查看新创建的 Release。

## 手动触发构建

如果需要手动触发构建（不创建 Release）：

1. 访问 GitHub 仓库的 Actions 页面
2. 选择 "Release" workflow
3. 点击 "Run workflow"
4. 选择分支并运行

## 支持的平台

自动构建会生成以下平台的二进制文件：

- **Linux x86_64**: `MediaClassifier-linux-x86_64.tar.gz`
- **Linux ARM64**: `MediaClassifier-linux-aarch64.tar.gz`
- **macOS x86_64** (Intel): `MediaClassifier-macos-x86_64.tar.gz`
- **macOS ARM64** (Apple Silicon): `MediaClassifier-macos-aarch64.tar.gz`
- **Windows x86_64**: `MediaClassifier-windows-x86_64.exe.zip`

## 版本号规范

遵循语义化版本 (Semantic Versioning)：

- **主版本号 (Major)**: 不兼容的 API 修改
- **次版本号 (Minor)**: 向下兼容的功能性新增
- **修订号 (Patch)**: 向下兼容的问题修正

示例：
- `v1.0.0` - 首个稳定版本
- `v1.1.0` - 新增功能
- `v1.1.1` - Bug 修复
- `v2.0.0` - 重大更新，可能不兼容旧版本

## 预发布版本

如果要创建预发布版本（beta、rc 等）：

```bash
git tag -a v0.2.0-beta.1 -m "Beta release"
git push origin v0.2.0-beta.1
```

GitHub Actions 会自动将其标记为 "Pre-release"。

## 故障排除

### 构建失败

1. 检查 Actions 页面的错误日志
2. 确保所有测试通过
3. 确保代码格式正确 (`cargo fmt`)
4. 确保没有 clippy 警告 (`cargo clippy`)

### Release 未创建

1. 确保标签以 `v` 开头（如 `v0.1.0`）
2. 检查 GitHub Token 权限
3. 查看 Actions 日志

### 下载的文件无法运行

**Linux/macOS**:
```bash
# 解压
tar xzf MediaClassifier-linux-x86_64.tar.gz

# 添加执行权限
chmod +x MediaClassifier

# 运行
./MediaClassifier
```

**Windows**:
1. 解压 zip 文件
2. 双击运行或在命令行中执行

## CI/CD 工作流说明

### CI (持续集成)
- 触发：推送到 main/master/develop 分支或 PR
- 功能：代码格式检查、Clippy 检查、运行测试、安全审计

### Build (构建)
- 触发：推送到 main/master 分支
- 功能：在三个主要平台上构建并上传 artifacts

### Release (发布)
- 触发：推送 v*.*.* 标签
- 功能：多平台编译、创建 GitHub Release、上传二进制文件
