use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize, Deserializer};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use std::os::unix::fs::PermissionsExt;

// 支持的操作系统和架构
#[derive(Debug)]
enum OsType {
    Darwin,
    Linux,
    Windows,
}

#[derive(Debug)]
enum ArchType {
    X64,
    Arm64,
    Arm,
    X86,
}

// 版本类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VersionType {
    Node,
    Rust,
    Python,
    Go,
}

impl std::fmt::Display for VersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionType::Node => write!(f, "Node.js"),
            VersionType::Rust => write!(f, "Rust"),
            VersionType::Python => write!(f, "Python"),
            VersionType::Go => write!(f, "Go"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeVersion {
    pub version: String,
    #[serde(deserialize_with = "deserialize_lts")]
    pub lts: bool,
    pub date: String,
    pub files: Vec<String>,
}

// Rust版本结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct RustVersion {
    pub version: String,
    pub date: String,
    pub stable: bool,
}

// 自定义反序列化函数来处理 lts 字段
fn deserialize_lts<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    
    match value {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::String(s) => {
            // 如果是字符串，可以根据内容判断
            // 这里简单地把任何非空字符串都视为 true
            Ok(!s.is_empty())
        }
        serde_json::Value::Null => Ok(false),
        _ => Ok(false), // 其他类型默认为 false
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Aliases {
    aliases: HashMap<String, String>,
}

// 自定义错误类型
#[derive(Debug)]
pub enum VersionError {
    NotInstalled(String, VersionType),
    NotFound(String, VersionType),
    CurrentlyActive(String, VersionType),
    IoError(io::Error),
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::NotInstalled(version, version_type) => 
                write!(f, "{} 版本 {} 未安装", version_type, version),
            VersionError::NotFound(version, version_type) => 
                write!(f, "找不到 {} 版本 {}", version_type, version),
            VersionError::CurrentlyActive(version, version_type) => 
                write!(f, "无法删除当前活动的 {} 版本 {}。请先切换到其他版本。", version_type, version),
            VersionError::IoError(err) => 
                write!(f, "IO错误: {}", err),
        }
    }
}

impl std::error::Error for VersionError {}

impl From<io::Error> for VersionError {
    fn from(err: io::Error) -> Self {
        VersionError::IoError(err)
    }
}

/// 版本管理器结构体，用于管理不同语言的版本
///
/// 支持管理Node.js和Rust版本，提供版本的安装、切换、删除等功能。
pub struct VersionManager {
    /// 基础目录，默认为~/.version-manager
    base_dir: PathBuf,
    /// 存放已安装版本的目录
    versions_dir: PathBuf,
    /// 别名配置文件路径
    aliases_file: PathBuf,
    /// 下载缓存目录
    cache_dir: PathBuf,
    /// 可执行文件目录
    bin_dir: PathBuf,
    /// 当前使用的版本
    current_version: Option<String>,
    /// 当前使用的版本类型
    current_version_type: VersionType,
    /// 操作系统类型
    os_type: OsType,
    /// 系统架构类型
    arch_type: ArchType,
}

impl VersionManager {
    /// 创建一个新的版本管理器实例
    ///
    /// 初始化必要的目录结构，检测系统环境，读取当前版本信息。
    ///
    /// # 返回
    ///
    /// 成功时返回VersionManager实例，失败时返回错误。
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .context("无法找到用户主目录")?
            .join(".version-manager");
        
        let versions_dir = base_dir.join("versions");
        let aliases_file = base_dir.join("aliases.json");
        let cache_dir = base_dir.join("cache");
        let bin_dir = base_dir.join("bin");
        
        // Create directories if they don't exist
        fs::create_dir_all(&base_dir).context("无法创建基础目录")?;
        fs::create_dir_all(&versions_dir).context("无法创建版本目录")?;
        fs::create_dir_all(&cache_dir).context("无法创建缓存目录")?;
        fs::create_dir_all(&bin_dir).context("无法创建bin目录")?;

        // Try to read current version from file
        let current_version = Self::read_current_version(&base_dir, VersionType::Node).ok();
        
        // Detect OS and architecture
        let os_type = Self::detect_os()?;
        let arch_type = Self::detect_arch()?;

        Ok(Self {
            base_dir,
            versions_dir,
            aliases_file,
            cache_dir,
            bin_dir,
            current_version,
            current_version_type: VersionType::Node,
            os_type,
            arch_type,
        })
    }

    /// 检测操作系统类型
    ///
    /// 根据系统环境变量OS来检测操作系统类型。
    ///
    /// # 返回
    ///
    /// 成功时返回OsType枚举值，失败时返回错误。
    fn detect_os() -> Result<OsType> {
        let os = env::consts::OS;
        match os {
            "macos" | "darwin" => Ok(OsType::Darwin),
            "linux" => Ok(OsType::Linux),
            "windows" => Ok(OsType::Windows),
            _ => Err(anyhow::anyhow!("不支持的操作系统: {}", os)),
        }
    }

    /// 检测架构类型
    ///
    /// 根据系统环境变量ARCH来检测架构类型。
    ///
    /// # 返回
    ///
    /// 成功时返回ArchType枚举值，失败时返回错误。
    fn detect_arch() -> Result<ArchType> {
        let arch = env::consts::ARCH;
        match arch {
            "x86_64" => Ok(ArchType::X64),
            "aarch64" => Ok(ArchType::Arm64),
            "arm" => Ok(ArchType::Arm),
            "x86" => Ok(ArchType::X86),
            _ => Err(anyhow::anyhow!("不支持的架构: {}", arch)),
        }
    }

    /// 获取操作系统和架构对应的下载 URL 后缀
    ///
    /// 根据操作系统类型和架构类型生成下载 URL 后缀。
    ///
    /// # 返回
    ///
    /// 成功时返回URL后缀字符串，失败时返回错误。
    fn get_os_arch_suffix(&self) -> String {
        match (&self.os_type, &self.arch_type) {
            (OsType::Darwin, ArchType::X64) => "darwin-x64".to_string(),
            (OsType::Darwin, ArchType::Arm64) => "darwin-arm64".to_string(),
            (OsType::Linux, ArchType::X64) => "linux-x64".to_string(),
            (OsType::Linux, ArchType::Arm64) => "linux-arm64".to_string(),
            (OsType::Linux, ArchType::Arm) => "linux-armv7l".to_string(),
            (OsType::Windows, ArchType::X64) => "win-x64".to_string(),
            (OsType::Windows, ArchType::X86) => "win-x86".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// 获取可执行文件的扩展名
    ///
    /// 根据操作系统类型获取可执行文件的扩展名。
    ///
    /// # 返回
    ///
    /// 成功时返回扩展名字符串，失败时返回错误。
    fn get_exe_extension(&self) -> &str {
        match self.os_type {
            OsType::Windows => ".exe",
            _ => "",
        }
    }

    /// 读取当前版本从文件
    ///
    /// 从指定目录下的.current-node文件读取当前版本信息。
    ///
    /// # 参数
    ///
    /// * `base_dir` - 基础目录
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回当前版本字符串，失败时返回错误。
    fn read_current_version(base_dir: &PathBuf, version_type: VersionType) -> Result<String> {
        let version_file = base_dir.join(format!(".current-{}", version_type));
        if version_file.exists() {
            let version = fs::read_to_string(version_file)?;
            Ok(version.trim().to_string())
        } else {
            Err(anyhow::anyhow!("找不到当前版本文件"))
        }
    }

    /// 保存当前版本到文件
    ///
    /// 将当前版本信息保存到指定目录下的.current-node文件。
    ///
    /// # 参数
    ///
    /// * `version` - 当前版本字符串
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    fn save_current_version(&self, version: &str, version_type: VersionType) -> Result<()> {
        let version_file = self.base_dir.join(format!(".current-{}", version_type));
        fs::write(version_file, version)?;
        Ok(())
    }

    /// 获取当前版本
    ///
    /// 获取当前使用的版本信息。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回当前版本字符串，失败时返回错误。
    pub fn get_current_version(&self, version_type: VersionType) -> Option<&String> {
        if self.current_version_type == version_type {
            self.current_version.as_ref()
        } else {
            None
        }
    }

    /// 读取别名配置
    ///
    /// 从指定目录下的aliases.json文件读取别名配置信息。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回别名配置信息，失败时返回错误。
    fn read_aliases(&self, version_type: VersionType) -> Result<Aliases> {
        let aliases_file = self.aliases_file.with_file_name(format!("aliases-{}.json", version_type));
        if !aliases_file.exists() {
            return Ok(Aliases {
                aliases: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&aliases_file)?;
        let aliases: Aliases = serde_json::from_str(&content)?;
        Ok(aliases)
    }

    /// 保存别名配置
    ///
    /// 将别名配置信息保存到指定目录下的aliases.json文件。
    ///
    /// # 参数
    ///
    /// * `aliases` - 别名配置信息
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    fn save_aliases(&self, aliases: &Aliases, version_type: VersionType) -> Result<()> {
        let aliases_file = self.aliases_file.with_file_name(format!("aliases-{}.json", version_type));
        let content = serde_json::to_string_pretty(aliases)?;
        fs::write(&aliases_file, content)?;
        Ok(())
    }

    /// 创建版本别名
    ///
    /// 为指定版本创建一个别名。
    ///
    /// # 参数
    ///
    /// * `alias` - 别名名称
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn create_alias(&self, alias: &str, version: &str, version_type: VersionType) -> Result<()> {
        // 检查版本是否已安装
        let version_dir = self.get_version_dir(version, version_type);
        if !version_dir.exists() {
            return Err(anyhow::anyhow!("{}", VersionError::NotInstalled(version.to_string(), version_type)));
        }

        let mut aliases = self.read_aliases(version_type)?;
        aliases.aliases.insert(alias.to_string(), version.to_string());
        self.save_aliases(&aliases, version_type)?;

        Ok(())
    }

    /// 获取别名对应的版本
    ///
    /// 获取指定别名对应的版本号。
    ///
    /// # 参数
    ///
    /// * `alias` - 别名名称
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回版本号字符串，失败时返回错误。
    pub fn get_alias(&self, alias: &str, version_type: VersionType) -> Result<Option<String>> {
        let aliases = self.read_aliases(version_type)?;
        Ok(aliases.aliases.get(alias).cloned())
    }

    /// 列出所有别名
    ///
    /// 列出所有已定义的别名。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回别名列表，失败时返回错误。
    pub fn list_aliases(&self, version_type: VersionType) -> Result<Vec<(String, String)>> {
        let aliases = self.read_aliases(version_type)?;
        let mut result = Vec::new();
        
        for (alias, version) in aliases.aliases {
            result.push((alias, version));
        }
        
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    /// 设置本地版本
    ///
    /// 在当前目录下创建一个文件指定使用的版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn set_local_version(&self, version: &str, version_type: VersionType) -> Result<()> {
        // 检查版本是否已安装
        let version_dir = self.get_version_dir(version, version_type);
        if !version_dir.exists() {
            return Err(anyhow::anyhow!("{}", VersionError::NotInstalled(version.to_string(), version_type)));
        }

        let current_dir = env::current_dir()?;
        let version_file = match version_type {
            VersionType::Node => current_dir.join(".node-version"),
            VersionType::Rust => current_dir.join(".rust-version"),
            VersionType::Python => current_dir.join(".python-version"),
            VersionType::Go => current_dir.join(".go-version"),
        };
        
        fs::write(version_file, version)?;
        
        Ok(())
    }

    /// 获取本地项目要求的版本
    ///
    /// 获取当前目录下指定的版本号。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回版本号字符串，失败时返回错误。
    #[allow(dead_code)]  // 标记为允许未使用
    pub fn get_local_version(version_type: VersionType) -> Result<Option<String>> {
        let current_dir = env::current_dir()?;
        let version_file = match version_type {
            VersionType::Node => current_dir.join(".node-version"),
            VersionType::Rust => current_dir.join(".rust-version"),
            VersionType::Python => current_dir.join(".python-version"),
            VersionType::Go => current_dir.join(".go-version"),
        };
        
        if version_file.exists() {
            let version = fs::read_to_string(version_file)?;
            Ok(Some(version.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    /// 使用指定版本执行命令
    ///
    /// 使用指定版本的环境执行命令。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `command` - 命令名称
    /// * `args` - 命令参数
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn exec_with_version(&self, version: &str, command: &str, args: &[String], version_type: VersionType) -> Result<()> {
        // 检查版本是否已安装，如果没有则安装
        let version_dir = self.get_version_dir(version, version_type);
        if !version_dir.exists() {
            println!("Version {} is not installed. Installing...", version);
            // 创建一个块作用域以避免 `?` 运算符立即返回
            {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(self.install_version(version, version_type))?;
            }
        }

        // 获取对应版本的二进制目录
        let bin_path = match version_type {
            VersionType::Node => version_dir.join(format!("node-v{}-{}/bin", version, self.get_os_arch_suffix())),
            VersionType::Rust => version_dir.join("bin"),
            VersionType::Python => version_dir.join("bin"),
            VersionType::Go => version_dir.join("bin"),
        };
        
        // 将该目录添加到 PATH 环境变量
        let path_var = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", bin_path.to_string_lossy(), path_var);
        
        // 执行命令
        let status = Command::new(command)
            .args(args)
            .env("PATH", new_path)
            .status()?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("命令执行失败，退出码: {}", status));
        }
        
        Ok(())
    }

    /// 清理缓存和临时文件
    ///
    /// 清理下载缓存和临时文件。
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn clean(&self) -> Result<()> {
        // 清理下载缓存
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir(&self.cache_dir)?;
        }
        
        // 查找并删除临时文件
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with("temp-") {
                    if path.is_file() {
                        fs::remove_file(path)?;
                    } else if path.is_dir() {
                        fs::remove_dir_all(path)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 自身更新
    ///
    /// 更新版本管理器自身。
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub async fn self_update(&self) -> Result<()> {
        // 这个功能的实现可能需要与特定的发布渠道集成
        // 这里简单地打印一条消息，实际应用中可以替换为真正的更新逻辑
        println!("Self-update functionality not yet implemented.");
        println!("Please manually update using cargo install --path .");
        Ok(())
    }

    /// 从其他版本管理器迁移
    ///
    /// 从其他版本管理器迁移已安装的版本。
    ///
    /// # 参数
    ///
    /// * `source` - 来源版本管理器名称
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回迁移的版本数量，失败时返回错误。
    pub async fn migrate_from(&self, source: &str, version_type: VersionType) -> Result<usize> {
        let mut migrated_count = 0;
        
        match (source.to_lowercase().as_str(), version_type) {
            ("nvm", VersionType::Node) => {
                // 尝试找到 NVM 安装目录
                let nvm_dir = if let Ok(dir) = env::var("NVM_DIR") {
                    PathBuf::from_str(&dir)?
                } else {
                    dirs::home_dir()
                        .context("Could not find home directory")?
                        .join(".nvm")
                };
                
                let versions_dir = nvm_dir.join("versions").join("node");
                
                if !versions_dir.exists() {
                    return Err(anyhow::anyhow!("找不到 NVM 版本目录"));
                }
                
                for entry in fs::read_dir(versions_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let version = entry.file_name().to_string_lossy().to_string();
                        // 跳过 "v" 前缀
                        let version = if version.starts_with('v') {
                            &version[1..]
                        } else {
                            &version
                        };
                        
                        // 检查是否已经安装
                        let target_dir = self.get_version_dir(version, version_type);
                        if !target_dir.exists() {
                            println!("Migrating Node.js version {} from NVM...", version);
                            // 复制文件
                            let source_dir = entry.path();
                            self.copy_dir_recursively(&source_dir, &target_dir)?;
                            migrated_count += 1;
                        }
                    }
                }
            },
            ("n", VersionType::Node) => {
                // 尝试找到 N 安装目录
                let n_prefix = env::var("N_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
                let n_versions_dir = PathBuf::from_str(&n_prefix)?.join("n").join("versions").join("node");
                
                if !n_versions_dir.exists() {
                    return Err(anyhow::anyhow!("找不到 N 版本目录"));
                }
                
                for entry in fs::read_dir(n_versions_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let version = entry.file_name().to_string_lossy().to_string();
                        
                        // 检查是否已经安装
                        let target_dir = self.get_version_dir(&version, version_type);
                        if !target_dir.exists() {
                            println!("Migrating Node.js version {} from N...", version);
                            // 复制文件
                            let source_dir = entry.path();
                            self.copy_dir_recursively(&source_dir, &target_dir)?;
                            migrated_count += 1;
                        }
                    }
                }
            },
            ("rustup", VersionType::Rust) => {
                // 尝试找到 rustup 安装目录
                let rustup_home = if let Ok(dir) = env::var("RUSTUP_HOME") {
                    PathBuf::from_str(&dir)?
                } else {
                    dirs::home_dir()
                        .context("Could not find home directory")?
                        .join(".rustup")
                };
                
                let toolchains_dir = rustup_home.join("toolchains");
                
                if !toolchains_dir.exists() {
                    return Err(anyhow::anyhow!("找不到 rustup 工具链目录"));
                }
                
                for entry in fs::read_dir(toolchains_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let toolchain = entry.file_name().to_string_lossy().to_string();
                        if toolchain.contains("stable") {
                            // 提取版本号
                            let version = if let Some(idx) = toolchain.find('-') {
                                toolchain[..idx].to_string()
                            } else {
                                toolchain.to_string()
                            };
                            
                            // 检查是否已经安装
                            let target_dir = self.get_version_dir(&version, version_type);
                            if !target_dir.exists() {
                                println!("Migrating Rust version {} from rustup...", version);
                                // 复制文件
                                let source_dir = entry.path();
                                self.copy_dir_recursively(&source_dir, &target_dir)?;
                                
                                // 创建bin目录
                                let bin_dir = target_dir.join("bin");
                                fs::create_dir_all(&bin_dir)?;
                                
                                // 复制可执行文件
                                let source_bin_dir = source_dir.join("bin");
                                if source_bin_dir.exists() {
                                    for bin_entry in fs::read_dir(&source_bin_dir)? {
                                        let bin_entry = bin_entry?;
                                        if bin_entry.file_type()?.is_file() {
                                            let file_name = bin_entry.file_name();
                                            let target_bin = bin_dir.join(&file_name);
                                            fs::copy(bin_entry.path(), &target_bin)?;
                                            
                                            // 设置执行权限
                                            if let OsType::Darwin | OsType::Linux = self.os_type {
                                                let mut perms = fs::metadata(&target_bin)?.permissions();
                                                perms.set_mode(0o755); // rwxr-xr-x
                                                fs::set_permissions(&target_bin, perms)?;
                                            }
                                        }
                                    }
                                }
                                
                                migrated_count += 1;
                            }
                        }
                    }
                }
            },
            _ => return Err(anyhow::anyhow!("不支持的源版本管理器: {} for {}", source, version_type)),
        }
        
        Ok(migrated_count)
    }

    /// 递归复制目录
    ///
    /// 递归复制源目录到目标目录。
    ///
    /// # 参数
    ///
    /// * `src` - 源目录
    /// * `dst` - 目标目录
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    fn copy_dir_recursively(&self, src: &Path, dst: &Path) -> Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if file_type.is_dir() {
                self.copy_dir_recursively(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            } else if file_type.is_symlink() {
                let target = fs::read_link(&src_path)?;
                std::os::unix::fs::symlink(target, &dst_path)?;
            }
        }
        
        Ok(())
    }

    /// 列出可用的版本
    ///
    /// 列出可用的版本信息。
    ///
    /// # 参数
    ///
    /// * `lts_only` - 是否只列出LTS版本
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回版本信息列表，失败时返回错误。
    pub async fn list_available_versions(&self, lts_only: bool, version_type: VersionType) -> Result<Vec<NodeVersion>> {
        match version_type {
            VersionType::Node => {
                let client = reqwest::Client::new();
                let response = client
                    .get("https://nodejs.org/dist/index.json")
                    .send()
                    .await?
                    .json::<Vec<NodeVersion>>()
                    .await?;

                let mut versions = if lts_only {
                    response.into_iter().filter(|v| v.lts).collect::<Vec<_>>()
                } else {
                    response
                };
                
                // 按版本号排序（从新到旧）
                versions.sort_by(|a, b| {
                    let a_parts: Vec<&str> = a.version.trim_start_matches('v').split('.').collect();
                    let b_parts: Vec<&str> = b.version.trim_start_matches('v').split('.').collect();
                    
                    for i in 0..std::cmp::min(a_parts.len(), b_parts.len()) {
                        let a_num = a_parts[i].parse::<i32>().unwrap_or(0);
                        let b_num = b_parts[i].parse::<i32>().unwrap_or(0);
                        
                        if a_num != b_num {
                            return b_num.cmp(&a_num); // 从新到旧排序
                        }
                    }
                    
                    b_parts.len().cmp(&a_parts.len())
                });

                Ok(versions)
            },
            VersionType::Rust => {
                // 获取Rust版本列表
                let client = reqwest::Client::new();
                let response = client
                    .get("https://static.rust-lang.org/dist/channel-rust-stable.toml")
                    .send()
                    .await?
                    .text()
                    .await?;
                
                // 简单解析TOML获取版本号
                let mut versions = Vec::new();
                let mut version = String::new();
                
                for line in response.lines() {
                    if line.starts_with("version = ") {
                        if let Some(v) = line.split('"').nth(1) {
                            version = v.to_string();
                            break;
                        }
                    }
                }
                
                if !version.is_empty() {
                    versions.push(NodeVersion {
                        version: version.clone(),
                        lts: true,
                        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        files: vec![],
                    });
                }
                
                // 获取其他版本
                if !lts_only {
                    let response = client
                        .get("https://static.rust-lang.org/dist/")
                        .send()
                        .await?
                        .text()
                        .await?;
                    
                    // 简单解析HTML获取版本号
                    for line in response.lines() {
                        if line.contains("rust-") && line.contains(".tar.gz") && !line.contains("beta") && !line.contains("nightly") {
                            if let Some(start) = line.find("rust-") {
                                if let Some(end) = line[start..].find(".tar.gz") {
                                    let v = &line[start + 5..start + end];
                                    if v.contains('-') {
                                        continue; // 跳过带有平台信息的文件
                                    }
                                    
                                    if !versions.iter().any(|existing: &NodeVersion| existing.version == v) {
                                        versions.push(NodeVersion {
                                            version: v.to_string(),
                                            lts: false,
                                            date: "".to_string(),
                                            files: vec![],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 按版本号排序
                versions.sort_by(|a, b| {
                    let a_parts: Vec<&str> = a.version.split('.').collect();
                    let b_parts: Vec<&str> = b.version.split('.').collect();
                    
                    for i in 0..std::cmp::min(a_parts.len(), b_parts.len()) {
                        let a_num = a_parts[i].parse::<i32>().unwrap_or(0);
                        let b_num = b_parts[i].parse::<i32>().unwrap_or(0);
                        
                        if a_num != b_num {
                            return b_num.cmp(&a_num); // 从新到旧排序
                        }
                    }
                    
                    b_parts.len().cmp(&a_parts.len())
                });
                
                Ok(versions)
            },
            VersionType::Python => {
                // 获取Python版本列表
                let client = reqwest::Client::new();
                let response = client
                    .get("https://www.python.org/ftp/python/")
                    .send()
                    .await?
                    .text()
                    .await?;
                
                // 简单解析HTML获取版本号
                let mut versions = Vec::new();
                for line in response.lines() {
                    if line.contains("href=\"") && line.contains("/\"") {
                        if let Some(start) = line.find("href=\"") {
                            if let Some(end) = line[start + 6..].find("\"") {
                                let version = &line[start + 6..start + 6 + end];
                                if version.ends_with('/') && version.chars().any(|c| c.is_digit(10)) {
                                    let version = version.trim_end_matches('/');
                                    if !versions.iter().any(|existing: &NodeVersion| existing.version == version) {
                                        versions.push(NodeVersion {
                                            version: version.to_string(),
                                            lts: false,
                                            date: "".to_string(),
                                            files: vec![],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 按版本号排序
                versions.sort_by(|a, b| {
                    let a_parts: Vec<&str> = a.version.split('.').collect();
                    let b_parts: Vec<&str> = b.version.split('.').collect();
                    
                    for i in 0..std::cmp::min(a_parts.len(), b_parts.len()) {
                        let a_num = a_parts[i].parse::<i32>().unwrap_or(0);
                        let b_num = b_parts[i].parse::<i32>().unwrap_or(0);
                        
                        if a_num != b_num {
                            return b_num.cmp(&a_num); // 从新到旧排序
                        }
                    }
                    
                    b_parts.len().cmp(&a_parts.len())
                });
                
                Ok(versions)
            },
            VersionType::Go => {
                // 获取Go版本列表
                let client = reqwest::Client::new();
                let response = client
                    .get("https://golang.org/dl/")
                    .send()
                    .await?
                    .text()
                    .await?;
                
                // 简单解析HTML获取版本号
                let mut versions = Vec::new();
                for line in response.lines() {
                    if line.contains("go") && line.contains("toggleVisible") {
                        if let Some(start) = line.find("go") {
                            if let Some(end) = line[start..].find(" ") {
                                let version = &line[start + 2..start + end];
                                if version.chars().any(|c| c.is_digit(10)) && !version.contains("beta") && !version.contains("rc") {
                                    if !versions.iter().any(|existing: &NodeVersion| existing.version == version) {
                                        versions.push(NodeVersion {
                                            version: version.to_string(),
                                            lts: false,
                                            date: "".to_string(),
                                            files: vec![],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 按版本号排序
                versions.sort_by(|a, b| {
                    let a_parts: Vec<&str> = a.version.split('.').collect();
                    let b_parts: Vec<&str> = b.version.split('.').collect();
                    
                    for i in 0..std::cmp::min(a_parts.len(), b_parts.len()) {
                        let a_num = a_parts[i].parse::<i32>().unwrap_or(0);
                        let b_num = b_parts[i].parse::<i32>().unwrap_or(0);
                        
                        if a_num != b_num {
                            return b_num.cmp(&a_num); // 从新到旧排序
                        }
                    }
                    
                    b_parts.len().cmp(&a_parts.len())
                });
                
                Ok(versions)
            }
        }
    }

    /// 安装最新版本
    ///
    /// 安装最新版本。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub async fn install_latest(&mut self, version_type: VersionType) -> Result<()> {
        let versions = self.list_available_versions(false, version_type).await?;
        
        if let Some(latest) = versions.first() {
            println!("Latest {} version: {}", version_type, latest.version);
            self.install_version(&latest.version, version_type).await?;
            Ok(())
        } else {
            return Err(anyhow::anyhow!("找不到最新的 {} 版本", version_type));
        }
    }

    /// 安装最新的LTS版本
    ///
    /// 安装最新的LTS版本。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub async fn install_latest_lts(&mut self, version_type: VersionType) -> Result<()> {
        let versions = self.list_available_versions(true, version_type).await?;
        
        if let Some(latest_lts) = versions.first() {
            println!("Latest LTS {} version: {}", version_type, latest_lts.version);
            self.install_version(&latest_lts.version, version_type).await?;
            Ok(())
        } else {
            return Err(anyhow::anyhow!("找不到最新的 LTS {} 版本", version_type));
        }
    }

    /// 安装指定版本
    ///
    /// 安装指定版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub async fn install_version(&self, version: &str, version_type: VersionType) -> Result<()> {
        let version_dir = self.get_version_dir(version, version_type);
        if version_dir.exists() {
            println!("Version {} is already installed", version);
            return Ok(());
        }

        // Create version directory
        fs::create_dir_all(&version_dir)?;

        // Determine appropriate URL based on OS and architecture
        let os_arch_suffix = match version_type {
            VersionType::Node => self.get_os_arch_suffix(),
            VersionType::Rust => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "x86_64-apple-darwin",
                    (OsType::Darwin, ArchType::Arm64) => "aarch64-apple-darwin",
                    (OsType::Linux, ArchType::X64) => "x86_64-unknown-linux-gnu",
                    (OsType::Linux, ArchType::Arm64) => "aarch64-unknown-linux-gnu",
                    (OsType::Linux, ArchType::Arm) => "linux-armv7l",
                    (OsType::Windows, ArchType::X64) => "x86_64-pc-windows-msvc",
                    (OsType::Windows, ArchType::X86) => "i686-pc-windows-msvc",
                    _ => "unknown",
                }.to_string()
            },
            VersionType::Python => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "macosx10.9.x86_64",
                    (OsType::Darwin, ArchType::Arm64) => "macos11.0.arm64",
                    (OsType::Linux, ArchType::X64) => "x86_64",
                    (OsType::Linux, ArchType::Arm64) => "aarch64",
                    (OsType::Linux, ArchType::Arm) => "armv7l",
                    (OsType::Windows, ArchType::X64) => "amd64",
                    (OsType::Windows, ArchType::X86) => "win32",
                    _ => "unknown",
                }.to_string()
            },
            VersionType::Go => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "darwin-amd64",
                    (OsType::Darwin, ArchType::Arm64) => "darwin-arm64",
                    (OsType::Linux, ArchType::X64) => "linux-amd64",
                    (OsType::Linux, ArchType::Arm64) => "linux-arm64",
                    (OsType::Linux, ArchType::Arm) => "linux-armv6l",
                    (OsType::Windows, ArchType::X64) => "windows-amd64",
                    (OsType::Windows, ArchType::X86) => "windows-386",
                    _ => "unknown",
                }.to_string()
            }
        };
        
        let extension = match self.os_type {
            OsType::Windows => ".zip",
            _ => ".tar.gz",
        };

        let url = match version_type {
            VersionType::Node => format!(
                "https://nodejs.org/dist/v{}/node-v{}-{}{}",
                version, version, os_arch_suffix, extension
            ),
            VersionType::Rust => format!(
                "https://static.rust-lang.org/dist/rust-{}-{}{}",
                version, os_arch_suffix, extension
            ),
            VersionType::Python => format!(
                "https://www.python.org/ftp/python/{}/Python-{}-{}.tar.xz",
                version, version, os_arch_suffix
            ),
            VersionType::Go => format!(
                "https://golang.org/dl/go{}.{}",
                version, os_arch_suffix
            ),
        };

        println!("Downloading {} v{} for {}...", version_type, version, os_arch_suffix);
        
        // Create a progress bar for download
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let total_size = response.content_length().unwrap_or(0);
        
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));
        
        // Download to a temporary file
        let temp_file = self.cache_dir.join(format!("{}{}", version, extension));
        let mut file = fs::File::create(&temp_file)?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        
        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }
        
        pb.finish_with_message(format!("Downloaded {} v{}", version_type, version));
        
        println!("Extracting...");
        
        // Extract based on the file type
        match extension {
            ".tar.gz" => {
                let file = fs::File::open(&temp_file)?;
                let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
                archive.unpack(&version_dir)?;
            },
            ".zip" => {
                let file = fs::File::open(&temp_file)?;
                let mut archive = zip::ZipArchive::new(file)?;
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let outpath = version_dir.join(file.name());
                    
                    if file.name().ends_with('/') {
                        fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                fs::create_dir_all(p)?;
                            }
                        }
                        let mut outfile = fs::File::create(&outpath)?;
                        io::copy(&mut file, &mut outfile)?;
                    }
                }
            },
            _ => return Err(anyhow::anyhow!("不支持的压缩文件格式: {}", extension)),
        }
        
        // 特殊处理Rust安装
        if version_type == VersionType::Rust {
            // 运行安装脚本
            let install_script = match self.os_type {
                OsType::Windows => version_dir.join(format!("rust-{}-{}/install.bat", version, os_arch_suffix)),
                _ => version_dir.join(format!("rust-{}-{}/install.sh", version, os_arch_suffix)),
            };
            
            if install_script.exists() {
                println!("Running Rust installation script...");
                
                let status = match self.os_type {
                    OsType::Windows => {
                        Command::new("cmd")
                            .arg("/C")
                            .arg(&install_script)
                            .arg("--prefix")
                            .arg(&version_dir)
                            .arg("--without=rust-docs")
                            .status()?
                    },
                    _ => {
                        Command::new("sh")
                            .arg(&install_script)
                            .arg("--prefix")
                            .arg(&version_dir)
                            .arg("--without=rust-docs")
                            .status()?
                    }
                };
                
                if !status.success() {
                    return Err(anyhow::anyhow!("Rust安装脚本执行失败，退出码: {}", status));
                }
            } else {
                println!("No installation script found, trying to set up manually...");
                // 手动设置bin目录
                let bin_dir = version_dir.join("bin");
                fs::create_dir_all(&bin_dir)?;
                
                // 查找并移动可执行文件
                let rust_bin_dir = match self.os_type {
                    OsType::Windows => version_dir.join(format!("rust-{}-{}/rustc/bin", version, os_arch_suffix)),
                    _ => version_dir.join(format!("rust-{}-{}/rustc/bin", version, os_arch_suffix)),
                };
                
                if rust_bin_dir.exists() {
                    for entry in fs::read_dir(&rust_bin_dir)? {
                        let entry = entry?;
                        if entry.file_type()?.is_file() {
                            let file_name = entry.file_name();
                            let target_bin = bin_dir.join(&file_name);
                            fs::copy(entry.path(), &target_bin)?;
                            
                            // 设置执行权限
                            if let OsType::Darwin | OsType::Linux = self.os_type {
                                let mut perms = fs::metadata(&target_bin)?.permissions();
                                perms.set_mode(0o755); // rwxr-xr-x
                                fs::set_permissions(&target_bin, perms)?;
                            }
                        }
                    }
                }
                
                // 复制cargo可执行文件
                let cargo_bin_dir = match self.os_type {
                    OsType::Windows => version_dir.join(format!("rust-{}-{}/cargo/bin", version, os_arch_suffix)),
                    _ => version_dir.join(format!("rust-{}-{}/cargo/bin", version, os_arch_suffix)),
                };
                
                if cargo_bin_dir.exists() {
                    for entry in fs::read_dir(&cargo_bin_dir)? {
                        let entry = entry?;
                        if entry.file_type()?.is_file() {
                            let file_name = entry.file_name();
                            let target_bin = bin_dir.join(&file_name);
                            fs::copy(entry.path(), &target_bin)?;
                            
                            // 设置执行权限
                            if let OsType::Darwin | OsType::Linux = self.os_type {
                                let mut perms = fs::metadata(&target_bin)?.permissions();
                                perms.set_mode(0o755); // rwxr-xr-x
                                fs::set_permissions(&target_bin, perms)?;
                            }
                        }
                    }
                }
            }
        }
        
        // 特殊处理Python安装
        if version_type == VersionType::Python {
            // 手动设置bin目录
            let bin_dir = version_dir.join("bin");
            fs::create_dir_all(&bin_dir)?;
            
            // 查找并移动可执行文件
            let python_bin_dir = match self.os_type {
                OsType::Windows => version_dir.join(format!("Python-{}-{}/python.exe", version, os_arch_suffix)),
                _ => version_dir.join(format!("Python-{}-{}/bin/python{}", version, os_arch_suffix, self.get_exe_extension())),
            };
            
            if python_bin_dir.exists() {
                let target_bin = bin_dir.join("python");
                fs::copy(python_bin_dir, &target_bin)?;
                
                // 设置执行权限
                if let OsType::Darwin | OsType::Linux = self.os_type {
                    let mut perms = fs::metadata(&target_bin)?.permissions();
                    perms.set_mode(0o755); // rwxr-xr-x
                    fs::set_permissions(&target_bin, perms)?;
                }
            }
        }
        
        // 特殊处理Go安装
        if version_type == VersionType::Go {
            // 手动设置bin目录
            let bin_dir = version_dir.join("bin");
            fs::create_dir_all(&bin_dir)?;
            
            // 查找并移动可执行文件
            let go_bin_dir = match self.os_type {
                OsType::Windows => version_dir.join(format!("go{}-{}.exe", version, os_arch_suffix)),
                _ => version_dir.join(format!("go{}-{}{}", version, os_arch_suffix, self.get_exe_extension())),
            };
            
            if go_bin_dir.exists() {
                let target_bin = bin_dir.join("go");
                fs::copy(go_bin_dir, &target_bin)?;
                
                // 设置执行权限
                if let OsType::Darwin | OsType::Linux = self.os_type {
                    let mut perms = fs::metadata(&target_bin)?.permissions();
                    perms.set_mode(0o755); // rwxr-xr-x
                    fs::set_permissions(&target_bin, perms)?;
                }
            }
        }
        
        // Set executable permissions for binaries on Unix-like systems
        if let OsType::Darwin | OsType::Linux = self.os_type {
            let bin_dir = match version_type {
                VersionType::Node => version_dir.join(format!("node-v{}-{}/bin", version, os_arch_suffix)),
                VersionType::Rust => version_dir.join("bin"),
                VersionType::Python => version_dir.join("bin"),
                VersionType::Go => version_dir.join("bin"),
            };
            if bin_dir.exists() {
                for entry in fs::read_dir(bin_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        let mut perms = fs::metadata(&path)?.permissions();
                        perms.set_mode(0o755); // rwxr-xr-x
                        fs::set_permissions(&path, perms)?;
                    }
                }
            }
        }

        println!("Successfully installed {} version {}", version_type, version);
        Ok(())
    }

    /// 使用指定版本
    ///
    /// 切换到指定版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn use_version(&mut self, version: &str, version_type: VersionType) -> Result<()> {
        let version_dir = self.get_version_dir(version, version_type);
        if !version_dir.exists() {
            return Err(anyhow::anyhow!("{}", VersionError::NotInstalled(version.to_string(), version_type)));
        }

        // Update symlinks
        fs::create_dir_all(&self.bin_dir)?;

        // Remove existing symlinks
        for entry in fs::read_dir(&self.bin_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_symlink() {
                fs::remove_file(entry.path())?;
            }
        }

        // Determine the bin directory based on OS and architecture
        let os_arch_suffix = match version_type {
            VersionType::Node => self.get_os_arch_suffix(),
            VersionType::Rust => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "x86_64-apple-darwin",
                    (OsType::Darwin, ArchType::Arm64) => "aarch64-apple-darwin",
                    (OsType::Linux, ArchType::X64) => "x86_64-unknown-linux-gnu",
                    (OsType::Linux, ArchType::Arm64) => "aarch64-unknown-linux-gnu",
                    (OsType::Linux, ArchType::Arm) => "linux-armv7l",
                    (OsType::Windows, ArchType::X64) => "x86_64-pc-windows-msvc",
                    (OsType::Windows, ArchType::X86) => "i686-pc-windows-msvc",
                    _ => "unknown",
                }.to_string()
            },
            VersionType::Python => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "macosx10.9.x86_64",
                    (OsType::Darwin, ArchType::Arm64) => "macos11.0.arm64",
                    (OsType::Linux, ArchType::X64) => "x86_64",
                    (OsType::Linux, ArchType::Arm64) => "aarch64",
                    (OsType::Linux, ArchType::Arm) => "armv7l",
                    (OsType::Windows, ArchType::X64) => "amd64",
                    (OsType::Windows, ArchType::X86) => "win32",
                    _ => "unknown",
                }.to_string()
            },
            VersionType::Go => {
                match (&self.os_type, &self.arch_type) {
                    (OsType::Darwin, ArchType::X64) => "darwin-amd64",
                    (OsType::Darwin, ArchType::Arm64) => "darwin-arm64",
                    (OsType::Linux, ArchType::X64) => "linux-amd64",
                    (OsType::Linux, ArchType::Arm64) => "linux-arm64",
                    (OsType::Linux, ArchType::Arm) => "linux-armv6l",
                    (OsType::Windows, ArchType::X64) => "windows-amd64",
                    (OsType::Windows, ArchType::X86) => "windows-386",
                    _ => "unknown",
                }.to_string()
            }
        };
        
        let bin_dir = match version_type {
            VersionType::Node => version_dir.join(format!("node-v{}-{}/bin", version, os_arch_suffix)),
            VersionType::Rust => version_dir.join("bin"),
            VersionType::Python => version_dir.join("bin"),
            VersionType::Go => version_dir.join("bin"),
        };
        
        // Create symlinks for all binaries in that directory
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let file_name = entry.file_name();
                    let target_path = self.bin_dir.join(&file_name);
                    
                    match self.os_type {
                        OsType::Windows => {
                            // 在 Windows 上，创建一个 .cmd 文件来启动相应的程序
                            let cmd_content = match version_type {
                                VersionType::Node => format!(
                                    "@echo off\r\n\"%~dp0\\..\\versions\\{}\\node-v{}-{}\\bin\\{}{}\" %*\r\n",
                                    version, version, os_arch_suffix, file_name.to_string_lossy(), self.get_exe_extension()
                                ),
                                VersionType::Rust => format!(
                                    "@echo off\r\n\"%~dp0\\..\\versions\\{}\\bin\\{}{}\" %*\r\n",
                                    version, file_name.to_string_lossy(), self.get_exe_extension()
                                ),
                                VersionType::Python => format!(
                                    "@echo off\r\n\"%~dp0\\..\\versions\\{}\\bin\\{}{}\" %*\r\n",
                                    version, file_name.to_string_lossy(), self.get_exe_extension()
                                ),
                                VersionType::Go => format!(
                                    "@echo off\r\n\"%~dp0\\..\\versions\\{}\\bin\\{}{}\" %*\r\n",
                                    version, file_name.to_string_lossy(), self.get_exe_extension()
                                ),
                            };
                            fs::write(target_path.with_extension("cmd"), cmd_content)?;
                        },
                        _ => {
                            // 在 Unix 系统上创建符号链接
                            std::os::unix::fs::symlink(entry.path(), target_path)?;
                        }
                    }
                }
            }
        } else {
            return Err(anyhow::anyhow!("找不到二进制目录"));
        }

        // Update PATH in shell config
        self.update_shell_config()?;

        // Save and update current version
        self.save_current_version(version, version_type)?;
        self.current_version = Some(version.to_string());
        self.current_version_type = version_type;

        println!("Switched to {} version {}", version_type, version);
        Ok(())
    }

    /// 列出已安装的版本
    ///
    /// 列出已安装的版本。
    ///
    /// # 参数
    ///
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回已安装版本列表，失败时返回错误。
    pub fn list_installed_versions(&self, _version_type: VersionType) -> Result<Vec<String>> {
        let mut versions = Vec::new();
        for entry in fs::read_dir(&self.versions_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    versions.push(name.to_string());
                }
            }
        }
        
        // 检查当前版本
        if let Some(current) = &self.current_version {
            for i in 0..versions.len() {
                if &versions[i] == current {
                    versions[i] = format!("{} (current)", versions[i]);
                    break;
                }
            }
        }
        
        Ok(versions)
    }

    /// 删除版本
    ///
    /// 删除指定版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn remove_version(&self, version: &str, version_type: VersionType) -> Result<()> {
        // Don't allow removing the current version
        if let Some(current) = &self.current_version {
            if current == version && self.current_version_type == version_type {
                return Err(anyhow::anyhow!("{}", VersionError::CurrentlyActive(version.to_string(), version_type)));
            }
        }

        let version_dir = self.get_version_dir(version, version_type);
        if !version_dir.exists() {
            return Err(anyhow::anyhow!("{}", VersionError::NotFound(version.to_string(), version_type)));
        }

        fs::remove_dir_all(version_dir).context(format!("删除 {} 版本 {} 失败", version_type, version))?;
        println!("成功删除 {} 版本 {}", version_type, version);
        Ok(())
    }

    /// 获取版本目录
    ///
    /// 获取指定版本的目录。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `version_type` - 版本类型
    ///
    /// # 返回
    ///
    /// 成功时返回版本目录，失败时返回错误。
    fn get_version_dir(&self, version: &str, version_type: VersionType) -> PathBuf {
        match version_type {
            VersionType::Node => self.versions_dir.join(version),
            VersionType::Rust => self.versions_dir.join(version),
            VersionType::Python => self.versions_dir.join(version),
            VersionType::Go => self.versions_dir.join(version),
        }
    }

    /// 更新shell配置
    ///
    /// 更新shell配置文件中的PATH环境变量。
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    fn update_shell_config(&self) -> Result<()> {
        let bin_path = self.bin_dir.to_string_lossy();
        
        match self.os_type {
            OsType::Windows => {
                // 在 Windows 上修改用户环境变量
                println!("请将以下目录添加到 PATH 环境变量中:");
                println!("{}", bin_path);
                println!("可以通过打开系统属性 -> 高级 -> 环境变量来实现。");
            },
            _ => {
                // 在 Unix 系统上修改 shell 配置文件
                let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
                let config_file = if shell.ends_with("zsh") {
                    dirs::home_dir()
                        .context("无法找到用户主目录")?
                        .join(".zshrc")
                } else {
                    dirs::home_dir()
                        .context("无法找到用户主目录")?
                        .join(".bashrc")
                };

                let export_line = format!("\nexport PATH=\"{}:$PATH\"\n", bin_path);
                
                if !config_file.exists() {
                    fs::write(&config_file, export_line)?;
                } else {
                    let content = fs::read_to_string(&config_file)?;
                    if !content.contains(&*bin_path) {
                        fs::write(&config_file, format!("{}{}", content, export_line))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取当前Rust版本
    ///
    /// 获取当前使用的Rust版本。
    ///
    /// # 返回
    ///
    /// 成功时返回当前Rust版本字符串，失败时返回错误。
    pub fn get_current_rust_version(&self) -> Option<&String> {
        if self.current_version_type == VersionType::Rust {
            self.current_version.as_ref()
        } else {
            None
        }
    }
    
    /// 列出可用的Rust版本
    ///
    /// 列出可用的Rust版本。
    ///
    /// # 参数
    ///
    /// * `stable_only` - 是否只列出稳定版本
    ///
    /// # 返回
    ///
    /// 成功时返回Rust版本列表，失败时返回错误。
    pub async fn list_available_rust_versions(&self, stable_only: bool) -> Result<Vec<String>> {
        let versions = self.list_available_versions(stable_only, VersionType::Rust).await?;
        let mut result = Vec::new();
        
        for version in versions {
            result.push(version.version);
        }
        
        Ok(result)
    }
    
    /// 安装Rust版本
    ///
    /// 安装指定的Rust版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub async fn install_rust_version(&self, version: &str) -> Result<()> {
        if version == "latest" {
            println!("安装最新的 Rust 版本...");
            let versions = self.list_available_rust_versions(true).await?;
            if let Some(latest) = versions.first() {
                self.install_version(latest, VersionType::Rust).await?;
            } else {
                return Err(anyhow::anyhow!("找不到最新的 Rust 版本"));
            }
        } else {
            self.install_version(version, VersionType::Rust).await?;
        }
        
        Ok(())
    }
    
    /// 使用指定的Rust版本
    ///
    /// 切换到指定的Rust版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn use_rust_version(&mut self, version: &str) -> Result<()> {
        self.use_version(version, VersionType::Rust)
    }
    
    /// 列出已安装的Rust版本
    ///
    /// 列出已安装的Rust版本。
    ///
    /// # 返回
    ///
    /// 成功时返回已安装Rust版本列表，失败时返回错误。
    pub fn list_installed_rust_versions(&self) -> Result<Vec<String>> {
        self.list_installed_versions(VersionType::Rust)
    }
    
    /// 删除Rust版本
    ///
    /// 删除指定的Rust版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn remove_rust_version(&self, version: &str) -> Result<()> {
        self.remove_version(version, VersionType::Rust)
    }
    
    /// 创建Rust版本别名
    ///
    /// 为指定的Rust版本创建一个别名。
    ///
    /// # 参数
    ///
    /// * `alias` - 别名名称
    /// * `version` - 版本号
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn create_rust_alias(&self, alias: &str, version: &str) -> Result<()> {
        self.create_alias(alias, version, VersionType::Rust)
    }
    
    /// 获取Rust别名对应的版本
    ///
    /// 获取指定Rust别名对应的版本。
    ///
    /// # 参数
    ///
    /// * `alias` - 别名名称
    ///
    /// # 返回
    ///
    /// 成功时返回Rust版本字符串，失败时返回错误。
    pub fn get_rust_alias(&self, alias: &str) -> Result<Option<String>> {
        self.get_alias(alias, VersionType::Rust)
    }
    
    /// 列出所有Rust别名
    ///
    /// 列出所有已定义的Rust别名。
    ///
    /// # 返回
    ///
    /// 成功时返回Rust别名列表，失败时返回错误。
    pub fn list_rust_aliases(&self) -> Result<Vec<(String, String)>> {
        self.list_aliases(VersionType::Rust)
    }
    
    /// 设置本地Rust版本
    ///
    /// 在当前目录下创建一个文件指定使用的Rust版本。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn set_local_rust_version(&self, version: &str) -> Result<()> {
        self.set_local_version(version, VersionType::Rust)
    }
    
    /// 使用指定Rust版本执行命令
    ///
    /// 使用指定的Rust版本执行命令。
    ///
    /// # 参数
    ///
    /// * `version` - 版本号
    /// * `command` - 命令名称
    /// * `args` - 命令参数
    ///
    /// # 返回
    ///
    /// 成功时返回Ok(()，失败时返回错误。
    pub fn exec_with_rust_version(&self, version: &str, command: &str, args: &[String]) -> Result<()> {
        self.exec_with_version(version, command, args, VersionType::Rust)
    }
    
    /// 从rustup迁移
    ///
    /// 从rustup迁移已安装的Rust版本。
    ///
    /// # 返回
    ///
    /// 成功时返回迁移的版本数量，失败时返回错误。
    #[allow(dead_code)]
    pub async fn migrate_from_rustup(&self) -> Result<usize> {
        self.migrate_from("rustup", VersionType::Rust).await
    }

    /// 获取可用的 Python 版本列表
    pub async fn list_available_python_versions(&self, stable_only: bool) -> Result<Vec<String>> {
        let versions = self.list_available_versions(false, VersionType::Python).await?;
        let mut result = Vec::new();
        
        for version in versions {
            // 如果只需要稳定版本，则跳过包含 alpha、beta、rc 的版本
            if stable_only && (version.version.contains("alpha") || 
                              version.version.contains("beta") || 
                              version.version.contains("rc")) {
                continue;
            }
            result.push(version.version);
        }
        
        Ok(result)
    }
    
    /// 安装指定的 Python 版本
    pub async fn install_python_version(&self, version: &str) -> Result<()> {
        // 直接使用版本字符串，不需要解析
        self.install_version(version, VersionType::Python).await?;
        Ok(())
    }
    
    /// 使用指定的 Python 版本
    pub fn use_python_version(&mut self, version: &str) -> Result<()> {
        self.use_version(version, VersionType::Python)
    }
    
    /// 获取当前使用的 Python 版本
    pub fn get_current_python_version(&self) -> Option<String> {
        self.get_current_version(VersionType::Python).cloned()
    }
    
    /// 列出已安装的 Python 版本
    pub fn list_installed_python_versions(&self) -> Result<Vec<String>> {
        self.list_installed_versions(VersionType::Python)
    }
    
    /// 删除指定的 Python 版本
    pub fn remove_python_version(&self, version: &str) -> Result<()> {
        self.remove_version(version, VersionType::Python)
    }
    
    /// 创建 Python 版本别名
    pub fn create_python_alias(&self, name: &str, version: &str) -> Result<()> {
        self.create_alias(name, version, VersionType::Python)
    }
    
    /// 获取 Python 版本别名对应的实际版本
    pub fn get_python_alias(&self, alias: &str) -> Result<Option<String>> {
        self.get_alias(alias, VersionType::Python)
    }
    
    /// 列出所有 Python 版本别名
    pub fn list_python_aliases(&self) -> Result<Vec<(String, String)>> {
        self.list_aliases(VersionType::Python)
    }
    
    /// 设置当前目录的 Python 版本
    pub fn set_local_python_version(&self, version: &str) -> Result<()> {
        self.set_local_version(version, VersionType::Python)
    }
    
    /// 使用指定的 Python 版本执行命令
    pub fn exec_with_python_version(&self, version: &str, command: &str, args: &[String]) -> Result<()> {
        self.exec_with_version(version, command, args, VersionType::Python)
    }
    
    /// 从 pyenv 迁移 Python 版本
    pub async fn migrate_from_pyenv(&self) -> Result<usize> {
        let pyenv_versions_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".pyenv")
            .join("versions");
        
        if !pyenv_versions_dir.exists() {
            return Ok(0);
        }
        
        let mut count = 0;
        for entry in fs::read_dir(pyenv_versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(version_str) = path.file_name().and_then(|n| n.to_str()) {
                    // 跳过非版本目录
                    if version_str.starts_with(".") {
                        continue;
                    }
                    
                    // 复制版本目录
                    let target_dir = self.versions_dir.join(version_str);
                    if !target_dir.exists() {
                        fs::create_dir_all(&target_dir)?;
                        
                        // 复制 bin 目录
                        let bin_dir = path.join("bin");
                        if bin_dir.exists() {
                            let target_bin_dir = target_dir.join("bin");
                            fs::create_dir_all(&target_bin_dir)?;
                            
                            for bin_entry in fs::read_dir(bin_dir)? {
                                let bin_entry = bin_entry?;
                                let bin_path = bin_entry.path();
                                
                                if bin_path.is_file() {
                                    let file_name = bin_path.file_name().unwrap();
                                    let target_bin_path = target_bin_dir.join(file_name);
                                    fs::copy(&bin_path, &target_bin_path)?;
                                    
                                    // 设置执行权限
                                    if let OsType::Darwin | OsType::Linux = self.os_type {
                                        let mut perms = fs::metadata(&target_bin_path)?.permissions();
                                        perms.set_mode(0o755); // rwxr-xr-x
                                        fs::set_permissions(&target_bin_path, perms)?;
                                    }
                                }
                            }
                            
                            count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    /// 获取可用的 Go 版本列表
    pub async fn list_available_go_versions(&self, stable_only: bool) -> Result<Vec<String>> {
        let versions = self.list_available_versions(false, VersionType::Go).await?;
        let mut result = Vec::new();
        
        for version in versions {
            // 如果只需要稳定版本，则跳过包含 beta、rc 的版本
            if stable_only && (version.version.contains("beta") || 
                              version.version.contains("rc")) {
                continue;
            }
            result.push(version.version);
        }
        
        Ok(result)
    }
    
    /// 安装指定的 Go 版本
    pub async fn install_go_version(&self, version: &str) -> Result<()> {
        // 直接使用版本字符串，不需要解析
        self.install_version(version, VersionType::Go).await?;
        Ok(())
    }
    
    /// 使用指定的 Go 版本
    pub fn use_go_version(&mut self, version: &str) -> Result<()> {
        self.use_version(version, VersionType::Go)
    }
    
    /// 获取当前使用的 Go 版本
    pub fn get_current_go_version(&self) -> Option<String> {
        self.get_current_version(VersionType::Go).cloned()
    }
    
    /// 列出已安装的 Go 版本
    pub fn list_installed_go_versions(&self) -> Result<Vec<String>> {
        self.list_installed_versions(VersionType::Go)
    }
    
    /// 删除指定的 Go 版本
    pub fn remove_go_version(&self, version: &str) -> Result<()> {
        self.remove_version(version, VersionType::Go)
    }
    
    /// 创建 Go 版本别名
    pub fn create_go_alias(&self, name: &str, version: &str) -> Result<()> {
        self.create_alias(name, version, VersionType::Go)
    }
    
    /// 获取 Go 版本别名对应的实际版本
    pub fn get_go_alias(&self, alias: &str) -> Result<Option<String>> {
        self.get_alias(alias, VersionType::Go)
    }
    
    /// 列出所有 Go 版本别名
    pub fn list_go_aliases(&self) -> Result<Vec<(String, String)>> {
        self.list_aliases(VersionType::Go)
    }
    
    /// 设置当前目录的 Go 版本
    pub fn set_local_go_version(&self, version: &str) -> Result<()> {
        self.set_local_version(version, VersionType::Go)
    }
    
    /// 使用指定的 Go 版本执行命令
    pub fn exec_with_go_version(&self, version: &str, command: &str, args: &[String]) -> Result<()> {
        self.exec_with_version(version, command, args, VersionType::Go)
    }
    
    /// 从 gvm 迁移 Go 版本
    pub async fn migrate_from_gvm(&self) -> Result<usize> {
        let gvm_versions_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".gvm")
            .join("gos");
        
        if !gvm_versions_dir.exists() {
            return Ok(0);
        }
        
        let mut count = 0;
        for entry in fs::read_dir(gvm_versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(version_str) = path.file_name().and_then(|n| n.to_str()) {
                    // 跳过非版本目录
                    if !version_str.starts_with("go") {
                        continue;
                    }
                    
                    // 提取版本号
                    let version = &version_str[2..]; // 去掉 "go" 前缀
                    
                    // 复制版本目录
                    let target_dir = self.versions_dir.join(version);
                    if !target_dir.exists() {
                        fs::create_dir_all(&target_dir)?;
                        
                        // 复制 bin 目录
                        let bin_dir = path.join("bin");
                        if bin_dir.exists() {
                            let target_bin_dir = target_dir.join("bin");
                            fs::create_dir_all(&target_bin_dir)?;
                            
                            for bin_entry in fs::read_dir(bin_dir)? {
                                let bin_entry = bin_entry?;
                                let bin_path = bin_entry.path();
                                
                                if bin_path.is_file() {
                                    let file_name = bin_path.file_name().unwrap();
                                    let target_bin_path = target_bin_dir.join(file_name);
                                    fs::copy(&bin_path, &target_bin_path)?;
                                    
                                    // 设置执行权限
                                    if let OsType::Darwin | OsType::Linux = self.os_type {
                                        let mut perms = fs::metadata(&target_bin_path)?.permissions();
                                        perms.set_mode(0o755); // rwxr-xr-x
                                        fs::set_permissions(&target_bin_path, perms)?;
                                    }
                                }
                            }
                            
                            count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(count)
    }
}
