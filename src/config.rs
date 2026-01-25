//! 配置管理模块
//!
//! 提供应用配置的加载、解析和默认值管理。
//! 配置文件采用 TOML 格式，支持从文件加载或使用内置默认值。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 应用配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 搜索配置
    pub search: SearchConfig,
    /// TUI 配置
    pub tui: TuiConfig,
    /// 存储配置
    pub storage: StorageConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 更新配置
    pub update: UpdateConfig,
}

/// HTTP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// 监听端口
    pub port: u16,
    /// 绑定地址
    pub bind: String,
    /// 最大上传文件大小（字节）
    pub max_upload_size: usize,
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// 默认搜索结果数量
    pub default_limit: usize,
    /// 最大搜索结果数量
    pub max_limit: usize,
    /// 索引写入缓冲区大小（字节）
    pub index_buffer_size: usize,
    /// 默认语言
    pub default_lang: String,
}

/// TUI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// 事件轮询超时（毫秒）
    pub poll_timeout_ms: u64,
    /// 日志缓冲区大小
    pub log_buffer_size: usize,
    /// 详情滚动步长
    pub scroll_step: u16,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// 数据目录（空表示使用默认路径）
    pub data_dir: Option<PathBuf>,
    /// 数据库文件名
    pub db_filename: String,
    /// 索引目录名
    pub index_dirname: String,
    /// 日志目录名
    pub log_dirname: String,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// 默认日志级别
    pub level: String,
    /// 调试模式日志级别
    pub debug_level: String,
}

/// 更新配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateConfig {
    /// tldr-pages GitHub API 地址（获取最新版本）
    pub github_api_url: String,
    /// tldr-pages 下载地址模板（{version} 会被替换为版本号）
    pub download_url_template: String,
    /// HTTP 请求 User-Agent
    pub user_agent: String,
    /// 备用版本号（API 不可用时使用）
    pub fallback_version: String,
    /// 允许导入的语言列表（空表示全部）
    pub languages: Vec<String>,
}

// 默认值实现



impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            bind: "127.0.0.1".to_string(),
            max_upload_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_limit: 20,
            max_limit: 100,
            index_buffer_size: 50_000_000,
            default_lang: "en".to_string(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            poll_timeout_ms: 100,
            log_buffer_size: 100,
            scroll_step: 1,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            db_filename: "data.redb".to_string(),
            index_dirname: "index".to_string(),
            log_dirname: "logs".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            debug_level: "debug,tantivy=info".to_string(),
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            github_api_url: "https://api.github.com/repos/tldr-pages/tldr/releases/latest".to_string(),
            download_url_template: "https://github.com/tldr-pages/tldr/archive/refs/tags/{version}.zip".to_string(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            fallback_version: "v2.3".to_string(),
            languages: vec!["en".to_string(), "zh".to_string()],
        }
    }
}

impl AppConfig {
    /// 从 TOML 文件加载配置
    /// 如果文件不存在，返回默认配置
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config file: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read config file: {}", e);
                }
            }
        }
        Self::default()
    }

    /// 从默认位置加载配置
    /// 优先级：
    /// 1. 当前目录下的 rtfm.toml
    /// 2. 数据目录下的 config.toml
    /// 3. 内置默认值
    pub fn load_default() -> Self {
        // 当前目录
        let current_config = PathBuf::from("rtfm.toml");
        if current_config.exists() {
            return Self::load(&current_config);
        }

        // 数据目录
        let data_dir = get_default_data_dir();
        let data_config = data_dir.join("config.toml");
        if data_config.exists() {
            return Self::load(&data_config);
        }

        // 默认值
        Self::default()
    }

    /// 获取数据目录
    pub fn get_data_dir(&self) -> PathBuf {
        self.storage
            .data_dir
            .clone()
            .or_else(|| std::env::var("RTFM_DATA_DIR").ok().map(PathBuf::from))
            .unwrap_or_else(get_default_data_dir)
    }

    /// 生成默认配置文件内容
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

/// 获取默认数据目录
fn get_default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rtfm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 3030);
        assert_eq!(config.server.bind, "127.0.0.1");
        assert_eq!(config.search.default_limit, 20);
        assert_eq!(config.search.max_limit, 100);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml();
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("port = 3030"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[server]
port = 8080
bind = "0.0.0.0"

[search]
default_limit = 50
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.bind, "0.0.0.0");
        assert_eq!(config.search.default_limit, 50);
        // 未指定的字段使用默认值
        assert_eq!(config.search.max_limit, 100);
    }
}
