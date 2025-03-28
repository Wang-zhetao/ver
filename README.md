# Ver - 高性能 Node.js 版本管理器

`ver` 是一个用 Rust 编写的快速、跨平台的 Node.js 版本管理工具，帮助你轻松管理多个 Node.js 版本。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## 特性

- 🚀 **高性能** - Rust 编写，启动迅速
- 🔄 **便捷切换** - 在不同的 Node.js 版本间轻松切换
- 🌐 **跨平台** - 支持 macOS、Linux 和 Windows
- 🏗️ **多架构** - 支持 x64、arm64 等多种架构
- 🏷️ **版本别名** - 为版本创建易记的别名
- 📁 **项目特定版本** - 为不同项目设置不同的 Node.js 版本
- 📦 **一键迁移** - 从其他版本管理器 (nvm, n) 迁移
- 🔍 **智能环境管理** - 自动处理环境变量和路径设置

## 安装

### 使用 Homebrew (macOS)

```bash
brew install ver
```

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/ver.git
cd ver

# 编译安装
cargo install --path .
```

### 预编译二进制文件

在 [Releases](https://github.com/yourusername/ver/releases) 页面下载对应平台的预编译二进制文件。

## 使用方法

### 基本命令

```bash
# 查看帮助
ver --help

# 列出可用的 Node.js 版本
ver list
ver list --lts  # 仅显示 LTS 版本

# 安装特定版本
ver install 18.17.0

# 安装最新版本或最新 LTS 版本
ver install latest
ver install lts

# 切换版本
ver use 18.17.0

# 查看当前使用的版本
ver current

# 列出已安装的版本
ver installed

# 删除特定版本
ver remove 18.17.0
```

### 版本别名

```bash
# 创建别名
ver alias myproject 18.17.0

# 使用别名切换版本
ver use myproject

# 列出所有别名
ver aliases
```

### 项目特定版本

```bash
# 为当前项目设置特定版本
ver local 16.13.0
```

这将在当前目录创建一个 `.node-version` 文件。

### 执行命令

无需切换全局版本，使用特定版本运行命令：

```bash
ver exec 14.17.0 npm install
```

### 迁移

从其他版本管理器迁移已安装的版本：

```bash
ver migrate nvm
ver migrate n
```

### 维护

```bash
# 清理缓存和临时文件
ver clean

# 更新 ver 自身
ver selfupdate
```

## 支持的平台

- **操作系统**: macOS, Linux, Windows
- **架构**: x64, arm64, x86, arm

## 开发

### 依赖

- Rust 1.70 或更高版本

### 构建

```bash
cargo build --release
```

## 贡献

欢迎提交 Pull Request 和 Issue。在提交 PR 前，请确保：

1. 代码通过 `cargo fmt` 和 `cargo clippy` 检查
2. 添加必要的测试
3. 更新相关文档

## 
