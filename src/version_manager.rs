use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize, Deserializer};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};
use tokio::process::Command as TokioCommand;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeVersion {
    pub version: String,
    #[serde(deserialize_with = "deserialize_lts")]
    pub lts: bool,
    pub date: String,
    pub files: Vec<String>,
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

pub struct VersionManager {
    base_dir: PathBuf,
    versions_dir: PathBuf,
    aliases_file: PathBuf,
    cache_dir: PathBuf,
    bin_dir: PathBuf,
    current_version: Option<String>,
    os_type: OsType,
    arch_type: ArchType,
}

impl VersionManager {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".node-version-manager");
        
        let versions_dir = base_dir.join("versions");
        let aliases_file = base_dir.join("aliases.json");
        let cache_dir = base_dir.join("cache");
        let bin_dir = base_dir.join("bin");
        
        // Create directories if they don't exist
        fs::create_dir_all(&base_dir)?;
        fs::create_dir_all(&versions_dir)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&bin_dir)?;

        // Try to read current version from file
        let current_version = Self::read_current_version(&base_dir).ok();

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
            os_type,
            arch_type,
        })
    }

    // 检测操作系统类型
    fn detect_os() -> Result<OsType> {
        let os = env::consts::OS;
        match os {
            "macos" | "darwin" => Ok(OsType::Darwin),
            "linux" => Ok(OsType::Linux),
            "windows" => Ok(OsType::Windows),
            _ => anyhow::bail!("Unsupported operating system: {}", os),
        }
    }

    // 检测架构类型
    fn detect_arch() -> Result<ArchType> {
        let arch = env::consts::ARCH;
        match arch {
            "x86_64" => Ok(ArchType::X64),
            "aarch64" => Ok(ArchType::Arm64),
            "arm" => Ok(ArchType::Arm),
            "x86" => Ok(ArchType::X86),
            _ => anyhow::bail!("Unsupported architecture: {}", arch),
        }
    }

    // 获取操作系统和架构对应的下载 URL 后缀
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

    // 获取可执行文件的扩展名
    fn get_exe_extension(&self) -> &str {
        match self.os_type {
            OsType::Windows => ".exe",
            _ => "",
        }
    }

    // Read the current version from a file
    fn read_current_version(base_dir: &PathBuf) -> Result<String> {
        let version_file = base_dir.join(".current");
        if version_file.exists() {
            let version = fs::read_to_string(version_file)?;
            Ok(version.trim().to_string())
        } else {
            anyhow::bail!("No current version set")
        }
    }

    // Save the current version to a file
    fn save_current_version(&self, version: &str) -> Result<()> {
        let version_file = self.base_dir.join(".current");
        fs::write(version_file, version)?;
        Ok(())
    }

    // Get the current version
    pub fn get_current_version(&self) -> Option<&String> {
        self.current_version.as_ref()
    }

    // 读取别名配置
    fn read_aliases(&self) -> Result<Aliases> {
        if !self.aliases_file.exists() {
            return Ok(Aliases {
                aliases: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.aliases_file)?;
        let aliases: Aliases = serde_json::from_str(&content)?;
        Ok(aliases)
    }

    // 保存别名配置
    fn save_aliases(&self, aliases: &Aliases) -> Result<()> {
        let content = serde_json::to_string_pretty(aliases)?;
        fs::write(&self.aliases_file, content)?;
        Ok(())
    }

    // 创建版本别名
    pub fn create_alias(&self, alias: &str, version: &str) -> Result<()> {
        // 检查版本是否已安装
        let version_dir = self.versions_dir.join(version);
        if !version_dir.exists() {
            anyhow::bail!("Version {} is not installed", version);
        }

        let mut aliases = self.read_aliases()?;
        aliases.aliases.insert(alias.to_string(), version.to_string());
        self.save_aliases(&aliases)?;

        Ok(())
    }

    // 获取别名对应的版本
    pub fn get_alias(&self, alias: &str) -> Result<Option<String>> {
        let aliases = self.read_aliases()?;
        Ok(aliases.aliases.get(alias).cloned())
    }

    // 列出所有别名
    pub fn list_aliases(&self) -> Result<Vec<(String, String)>> {
        let aliases = self.read_aliases()?;
        let mut result = Vec::new();
        
        for (alias, version) in aliases.aliases {
            result.push((alias, version));
        }
        
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    // 设置本地版本
    pub fn set_local_version(&self, version: &str) -> Result<()> {
        // 检查版本是否已安装
        let version_dir = self.versions_dir.join(version);
        if !version_dir.exists() {
            anyhow::bail!("Version {} is not installed", version);
        }

        let current_dir = env::current_dir()?;
        let node_version_file = current_dir.join(".node-version");
        
        fs::write(node_version_file, version)?;
        
        Ok(())
    }

    // 获取本地项目要求的版本
    pub fn get_local_version() -> Result<Option<String>> {
        let current_dir = env::current_dir()?;
        let node_version_file = current_dir.join(".node-version");
        
        if node_version_file.exists() {
            let version = fs::read_to_string(node_version_file)?;
            Ok(Some(version.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    // 使用指定版本执行命令
    pub fn exec_with_version(&self, version: &str, command: &str, args: &[String]) -> Result<()> {
        // 检查版本是否已安装，如果没有则安装
        let version_dir = self.versions_dir.join(version);
        if !version_dir.exists() {
            println!("Version {} is not installed. Installing...", version);
            // 创建一个块作用域以避免 `?` 运算符立即返回
            {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(self.install_version(version))?;
            }
        }

        // 获取对应版本的二进制目录
        let bin_path = version_dir.join(format!("node-v{}-{}/bin", version, self.get_os_arch_suffix()));
        
        // 将该目录添加到 PATH 环境变量
        let path_var = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", bin_path.to_string_lossy(), path_var);
        
        // 执行命令
        let status = Command::new(command)
            .args(args)
            .env("PATH", new_path)
            .status()?;
            
        if !status.success() {
            anyhow::bail!("Command failed with exit code: {}", status);
        }
        
        Ok(())
    }

    // 清理缓存和临时文件
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

    // 自身更新
    pub async fn self_update(&self) -> Result<()> {
        // 这个功能的实现可能需要与特定的发布渠道集成
        // 这里简单地打印一条消息，实际应用中可以替换为真正的更新逻辑
        println!("Self-update functionality not yet implemented.");
        println!("Please manually update using cargo install --path .");
        Ok(())
    }

    // 从其他版本管理器迁移
    pub async fn migrate_from(&self, source: &str) -> Result<usize> {
        let mut migrated_count = 0;
        
        match source.to_lowercase().as_str() {
            "nvm" => {
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
                    anyhow::bail!("Cannot find NVM versions directory at {}", versions_dir.display());
                }
                
                for entry in fs::read_dir(versions_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let version_str = entry.file_name().to_string_lossy();
                        // 跳过 "v" 前缀
                        let version = if version_str.starts_with('v') {
                            &version_str[1..]
                        } else {
                            &version_str
                        };
                        
                        // 检查是否已经安装
                        let target_dir = self.versions_dir.join(version);
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
            "n" => {
                // 尝试找到 N 安装目录
                let n_prefix = env::var("N_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
                let n_versions_dir = PathBuf::from_str(&n_prefix)?.join("n").join("versions").join("node");
                
                if !n_versions_dir.exists() {
                    anyhow::bail!("Cannot find N versions directory at {}", n_versions_dir.display());
                }
                
                for entry in fs::read_dir(n_versions_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let version = entry.file_name().to_string_lossy();
                        
                        // 检查是否已经安装
                        let target_dir = self.versions_dir.join(&*version);
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
            _ => anyhow::bail!("Unsupported source version manager: {}", source),
        }
        
        Ok(migrated_count)
    }

    // 递归复制目录
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

    pub async fn list_available_versions(&self, lts_only: bool) -> Result<Vec<NodeVersion>> {
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

    // 安装最新版本
    pub async fn install_latest(&mut self) -> Result<()> {
        let versions = self.list_available_versions(false).await?;
        
        if let Some(latest) = versions.first() {
            println!("Latest Node.js version: {}", latest.version);
            self.install_version(&latest.version).await?;
            Ok(())
        } else {
            anyhow::bail!("Failed to find the latest Node.js version")
        }
    }

    // 安装最新 LTS 版本
    pub async fn install_latest_lts(&mut self) -> Result<()> {
        let versions = self.list_available_versions(true).await?;
        
        if let Some(latest_lts) = versions.first() {
            println!("Latest LTS Node.js version: {}", latest_lts.version);
            self.install_version(&latest_lts.version).await?;
            Ok(())
        } else {
            anyhow::bail!("Failed to find the latest LTS Node.js version")
        }
    }

    pub async fn install_version(&self, version: &str) -> Result<()> {
        let version_dir = self.versions_dir.join(version);
        if version_dir.exists() {
            println!("Version {} is already installed", version);
            return Ok(());
        }

        // Create version directory
        fs::create_dir_all(&version_dir)?;

        // Determine appropriate URL based on OS and architecture
        let os_arch_suffix = self.get_os_arch_suffix();
        let extension = match self.os_type {
            OsType::Windows => ".zip",
            _ => ".tar.gz",
        };

        let url = format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}{}",
            version, version, os_arch_suffix, extension
        );

        println!("Downloading Node.js v{} for {}...", version, os_arch_suffix);
        
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
        
        pb.finish_with_message(format!("Downloaded Node.js v{}", version));
        
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
            _ => anyhow::bail!("Unsupported archive format: {}", extension),
        }
        
        // Set executable permissions for binaries on Unix-like systems
        if let OsType::Darwin | OsType::Linux = self.os_type {
            let bin_dir = version_dir.join(format!("node-v{}-{}/bin", version, os_arch_suffix));
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

        println!("Successfully installed Node.js version {}", version);
        Ok(())
    }

    pub fn use_version(&mut self, version: &str) -> Result<()> {
        let version_dir = self.versions_dir.join(version);
        if !version_dir.exists() {
            anyhow::bail!("Version {} is not installed", version);
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
        let os_arch_suffix = self.get_os_arch_suffix();
        let node_bin_dir = version_dir.join(format!("node-v{}-{}/bin", version, os_arch_suffix));
        
        // Create symlinks for all binaries in that directory
        for entry in fs::read_dir(&node_bin_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let file_name = entry.file_name();
                let target_path = self.bin_dir.join(&file_name);
                
                match self.os_type {
                    OsType::Windows => {
                        // 在 Windows 上，创建一个 .cmd 文件来启动相应的程序
                        let cmd_content = format!(
                            "@echo off\r\n\"%~dp0\\..\\versions\\{}\\node-v{}-{}\\bin\\{}{}\" %*\r\n",
                            version, version, os_arch_suffix, file_name.to_string_lossy(), self.get_exe_extension()
                        );
                        fs::write(target_path.with_extension("cmd"), cmd_content)?;
                    },
                    _ => {
                        // 在 Unix 系统上创建符号链接
                        std::os::unix::fs::symlink(entry.path(), target_path)?;
                    }
                }
            }
        }

        // Update PATH in shell config
        self.update_shell_config()?;

        // Save and update current version
        self.save_current_version(version)?;
        self.current_version = Some(version.to_string());

        println!("Switched to Node.js version {}", version);
        Ok(())
    }

    pub fn list_installed_versions(&self) -> Result<Vec<String>> {
        let mut versions = Vec::new();
        for entry in fs::read_dir(&self.versions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let version = entry.file_name().to_string_lossy().to_string();
                if let Some(current) = &self.current_version {
                    if &version == current {
                        versions.push(format!("{} (current)", version));
                        continue;
                    }
                }
                versions.push(version);
            }
        }
        versions.sort();
        Ok(versions)
    }

    pub fn remove_version(&self, version: &str) -> Result<()> {
        // Don't allow removing the current version
        if let Some(current) = &self.current_version {
            if current == version {
                anyhow::bail!("Cannot remove the currently active version. Switch to another version first.");
            }
        }

        let version_dir = self.versions_dir.join(version);
        if !version_dir.exists() {
            anyhow::bail!("Version {} is not installed", version);
        }

        fs::remove_dir_all(version_dir)?;
        println!("Successfully removed Node.js version {}", version);
        Ok(())
    }

    fn update_shell_config(&self) -> Result<()> {
        let bin_path = self.bin_dir.to_string_lossy();
        
        match self.os_type {
            OsType::Windows => {
                // 在 Windows 上修改用户环境变量
                println!("Please add the following directory to your PATH environment variable:");
                println!("{}", bin_path);
                println!("You can do this by opening System Properties -> Advanced -> Environment Variables.");
            },
            _ => {
                // 在 Unix 系统上修改 shell 配置文件
                let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
                let config_file = if shell.ends_with("zsh") {
                    dirs::home_dir()
                        .context("Could not find home directory")?
                        .join(".zshrc")
                } else {
                    dirs::home_dir()
                        .context("Could not find home directory")?
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
} 
