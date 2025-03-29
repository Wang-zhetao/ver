# Ver-dev - 高性能版本管理器

`ver-dev` 是一个用 Rust 编写的快速、跨平台的版本管理工具，帮助你轻松管理多个 Node.js、Rust、Python 和 Go 版本。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## 特性

- 🚀 **高性能**: 使用 Rust 编写，速度快，资源占用少
- 🔄 **多语言支持**: 管理 Node.js、Rust、Python 和 Go 版本
- 🌈 **彩色输出**: 直观的彩色终端输出，区分不同语言
- 🔄 **版本切换**: 快速在不同版本之间切换
- 📦 **简单安装**: 无需额外依赖，一键安装
- 🔌 **跨平台**: 支持 macOS、Linux 和 Windows
- 🏷️ **版本别名**: 为常用版本创建别名
- 📂 **项目特定版本**: 为不同项目设置不同的版本
- 🔄 **迁移工具**: 从其他版本管理器（如 nvm、rustup、pyenv、gvm）迁移

## 安装

### 使用 Homebrew (macOS)

```bash
brew install ver-dev
```

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/ver.git
cd ver

# 编译安装
cargo install --path .
```

### 开发版本安装

如果你想使用最新的开发版本，可以创建一个符号链接：

```bash
# 构建开发版本
cargo b

# 创建符号链接到 ~/.cargo/bin 目录
ln -sf "$(pwd)/target/debug/ver" ~/.cargo/bin/ver-dev

# 现在可以使用 ver-dev 命令
ver-dev -h
```

### 预编译二进制文件

在 [Releases](https://github.com/yourusername/ver/releases) 页面下载对应平台的预编译二进制文件。

## 使用方法

### Node.js 版本管理

```bash
# 查看帮助
ver-dev --help

# 列出可用的 Node.js 版本
ver-dev list
ver-dev list --lts  # 仅显示 LTS 版本

# 安装特定版本
ver-dev install 18.17.0

# 安装最新版本或最新 LTS 版本
ver-dev install latest
ver-dev install lts

# 切换版本
ver-dev use 18.17.0

# 查看当前使用的版本
ver-dev current

# 列出已安装的版本
ver-dev installed

# 删除特定版本
ver-dev remove 18.17.0
```

### Rust 版本管理

```bash
# 列出可用的 Rust 版本
ver-dev rust list
ver-dev rust list --stable  # 仅显示稳定版本

# 安装特定 Rust 版本
ver-dev rust install 1.85.0

# 安装最新版本
ver-dev rust install latest
ver-dev rust install stable  # 安装最新稳定版

# 切换 Rust 版本
ver-dev rust use 1.85.0

# 查看当前使用的 Rust 版本
ver-dev rust current

# 列出已安装的 Rust 版本
ver-dev rust installed

# 删除特定 Rust 版本
ver-dev rust remove 1.85.0
```

### Python 版本管理

```bash
# 列出可用的 Python 版本
ver-dev python list
ver-dev python list --stable  # 仅显示稳定版本

# 安装特定 Python 版本
ver-dev python install 3.12.0

# 安装最新版本
ver-dev python install latest
ver-dev python install stable  # 安装最新稳定版

# 切换 Python 版本
ver-dev python use 3.12.0

# 查看当前使用的 Python 版本
ver-dev python current

# 列出已安装的 Python 版本
ver-dev python installed

# 删除特定 Python 版本
ver-dev python remove 3.12.0

# 创建 Python 别名
ver-dev python alias myproject 3.12.0

# 使用别名切换版本
ver-dev python use myproject

# 列出所有 Python 别名
ver-dev python aliases

# 为当前项目设置特定 Python 版本
ver-dev python local 3.12.0

# 从 pyenv 迁移 Python 版本
ver-dev python migrate pyenv
```

### Go 版本管理

```bash
# 列出可用的 Go 版本
ver-dev go list
ver-dev go list --stable  # 仅显示稳定版本

# 安装特定 Go 版本
ver-dev go install 1.22.0

# 安装最新版本
ver-dev go install latest
ver-dev go install stable  # 安装最新稳定版

# 切换 Go 版本
ver-dev go use 1.22.0

# 查看当前使用的 Go 版本
ver-dev go current

# 列出已安装的 Go 版本
ver-dev go installed

# 删除特定 Go 版本
ver-dev go remove 1.22.0

# 创建 Go 别名
ver-dev go alias myproject 1.22.0

# 使用别名切换版本
ver-dev go use myproject

# 列出所有 Go 别名
ver-dev go aliases

# 为当前项目设置特定 Go 版本
ver-dev go local 1.22.0

# 从 gvm 迁移 Go 版本
ver-dev go migrate gvm
```

### 版本别名

```bash
# 创建 Node.js 别名
ver-dev alias myproject 18.17.0

# 创建 Rust 别名
ver-dev rust alias myproject 1.85.0

# 使用别名切换版本
ver-dev use myproject
ver-dev rust use myproject

# 列出所有别名
ver-dev aliases
ver-dev rust aliases
```

### 项目特定版本

```bash
# 为当前项目设置特定 Node.js 版本
ver-dev local 16.13.0

# 为当前项目设置特定 Rust 版本
ver-dev rust local 1.85.0
```

这将在当前目录创建一个 `.node-version` 或 `.rust-version` 文件。

### 执行命令

无需切换全局版本，使用特定版本运行命令：

```bash
# 使用特定 Node.js 版本运行命令
ver-dev exec 14.17.0 npm install

# 使用特定 Rust 版本运行命令
ver-dev rust exec 1.85.0 cargo b
```

### 迁移

从其他版本管理器迁移已安装的版本：

```bash
# 从 nvm 迁移 Node.js 版本
ver-dev migrate nvm

# 从 rustup 迁移 Rust 版本
ver-dev rust migrate rustup
```

### 维护

```bash
# 清理缓存和临时文件
ver-dev clean

# 更新 ver 自身
ver-dev selfupdate
```

## 彩色终端输出

为了提高可读性和用户体验，ver-dev 使用彩色终端输出来区分不同的语言和版本信息：

- **Node.js**: 绿色
- **Rust**: 黄色
- **Python**: 蓝色
- **Go**: 红色

当前版本和重要信息会以粗体显示，使您可以更容易地识别关键信息。

## 支持的平台

- **操作系统**: macOS, Linux, Windows
- **架构**: x64, arm64, x86, arm

## 开发

### 依赖

- Rust 1.70 或更高版本

### 构建

```bash
# 使用 Cargo 构建（开发版本）
cargo b

# 构建发布版本
cargo b --release

# 运行测试
cargo t

# 检查代码风格
cargo fmt

# 代码静态分析
cargo c
```

## 贡献

欢迎提交 Pull Request 和 Issue。在提交 PR 前，请确保：

1. 代码通过 `cargo fmt` 和 `cargo clippy` 检查
2. 添加必要的测试
3. 更新相关文档
