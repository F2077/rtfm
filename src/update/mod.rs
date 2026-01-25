use std::io::{Cursor, Read};

use flate2::read::GzDecoder;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::Deserialize;
use tar::Archive;
use thiserror::Error;
use zip::ZipArchive;

use crate::config::UpdateConfig;
use crate::storage::{Command, Example};

/// GitHub Release 信息
#[derive(Debug)]
pub struct ReleaseInfo {
  pub tag_name: String,
  pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
  tag_name: String,
}

/// 检查 GitHub 最新版本
pub async fn check_github_release(config: &UpdateConfig) -> anyhow::Result<ReleaseInfo> {
  let client = reqwest::Client::builder()
    .user_agent(&config.user_agent)
    .build()?;

  // 尝试获取最新版本
  let response = client
    .get(&config.github_api_url)
    .header("Accept", "application/vnd.github.v3+json")
    .send()
    .await;

  let tag_name = match response {
    Ok(resp) if resp.status().is_success() => {
      let release: GithubRelease = resp.json().await?;
      release.tag_name
    }
    _ => {
      // API 限制时使用备用版本
      config.fallback_version.clone()
    }
  };

  // 使用配置的下载地址模板
  let download_url = Some(config.download_url_template.replace("{version}", &tag_name));

  Ok(ReleaseInfo {
    tag_name: tag_name.trim_start_matches('v').to_string(),
    download_url,
  })
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// 解析 tldr-pages 压缩包
pub fn parse_tldr_archive(data: &[u8]) -> Result<Vec<Command>, UpdateError> {
    // 尝试作为 ZIP 解析
    if let Ok(commands) = parse_zip_archive(data) {
        return Ok(commands);
    }

    // 尝试作为 tar.gz 解析
    if let Ok(commands) = parse_targz_archive(data) {
        return Ok(commands);
    }

    Err(UpdateError::Parse("Unrecognized archive format".to_string()))
}

fn parse_zip_archive(data: &[u8]) -> Result<Vec<Command>, UpdateError> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    let mut commands = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // 只处理 .md 文件
        if !name.ends_with(".md") {
            continue;
        }

        // 解析路径以获取语言和平台
        let (lang, platform, cmd_name) = match parse_tldr_path(&name) {
            Some(info) => info,
            None => continue,
        };

        // 读取内容
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // 解析 Markdown
        if let Some(cmd) = parse_tldr_markdown(&content, cmd_name, lang, platform) {
            commands.push(cmd);
        }
    }

    Ok(commands)
}

fn parse_targz_archive(data: &[u8]) -> Result<Vec<Command>, UpdateError> {
    let cursor = Cursor::new(data);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    let mut commands = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();

        // 只处理 .md 文件
        if !path.ends_with(".md") {
            continue;
        }

        // 解析路径以获取语言和平台
        let (lang, platform, cmd_name) = match parse_tldr_path(&path) {
            Some(info) => info,
            None => continue,
        };

        // 读取内容
        let mut content = String::new();
        entry.read_to_string(&mut content)?;

        // 解析 Markdown
        if let Some(cmd) = parse_tldr_markdown(&content, cmd_name, lang, platform) {
            commands.push(cmd);
        }
    }

    Ok(commands)
}

/// 从 tldr-pages 路径解析语言、平台和命令名
/// 例如: pages.zh/common/docker.md -> ("zh", "common", "docker")
fn parse_tldr_path(path: &str) -> Option<(String, String, String)> {
    let path = path.replace('\\', "/");
    let parts: Vec<&str> = path.split('/').collect();

    // 查找 pages 目录
    let pages_idx = parts.iter().position(|p| p.starts_with("pages"))?;

    if parts.len() < pages_idx + 3 {
        return None;
    }

    // 解析语言
    let lang_part = parts[pages_idx];
    let lang = if lang_part.contains('.') {
        lang_part.split('.').nth(1).unwrap_or("en")
    } else {
        "en"
    };

    // 平台 (common, linux, windows, etc.)
    let platform = parts[pages_idx + 1];

    // 命令名 (去掉 .md 扩展名)
    let filename = parts.last()?;
    let cmd_name = filename.trim_end_matches(".md");

    Some((lang.to_string(), platform.to_string(), cmd_name.to_string()))
}

/// 解析 tldr 格式的 Markdown 文件
/// tldr 格式：
/// # command-name
/// > Description.
/// > More information: <url>.
/// - Example description:
/// `code`
fn parse_tldr_markdown(content: &str, name: String, lang: String, platform: String) -> Option<Command> {
    let parser = Parser::new(content);

    let mut description = String::new();
    let mut examples: Vec<Example> = Vec::new();
    let mut current_example_desc = String::new();
    let mut current_code = String::new();
    let mut in_heading = false;
    let mut in_list_item = false;
    let mut in_code_block = false;
    let mut in_blockquote = false;
    let mut heading_level = 0;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = level as u8;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
            }
            Event::Start(Tag::BlockQuote) => {
                in_blockquote = true;
            }
            Event::End(TagEnd::BlockQuote) => {
                in_blockquote = false;
            }
            Event::Start(Tag::Item) => {
                in_list_item = true;
            }
            Event::End(TagEnd::Item) => {
                // 列表项结束时，保存描述（等待后续代码块）
                in_list_item = false;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                current_code.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                // 代码块结束时，与之前的描述配对创建示例
                if !current_code.is_empty() {
                    let desc = if current_example_desc.is_empty() {
                        "Example".to_string()
                    } else {
                        // 清理描述：去掉末尾的冒号
                        current_example_desc.trim().trim_end_matches(':').to_string()
                    };
                    examples.push(Example {
                        description: desc,
                        code: current_code.trim().to_string(),
                    });
                    current_example_desc.clear();
                }
                in_code_block = false;
            }
            Event::Text(text) => {
                if in_heading && heading_level == 1 {
                    // 跳过标题（命令名）
                } else if in_code_block {
                    // 收集代码块内容
                    current_code.push_str(&text);
                } else if in_list_item {
                    // 收集示例描述
                    current_example_desc.push_str(&text);
                } else if in_blockquote {
                    // 处理引用块中的描述
                    let text = text.trim();
                    if !text.is_empty() 
                        && !text.contains("More information") 
                        && !text.contains("更多信息")
                        && !text.starts_with("http")
                        && !text.starts_with('<')
                    {
                        if description.is_empty() {
                            description = text.to_string();
                        } else {
                            // 多行描述
                            description.push(' ');
                            description.push_str(text);
                        }
                    }
                }
            }
            Event::Code(code) => {
                // 行内代码（在列表项中）
                if in_list_item {
                    current_example_desc.push('`');
                    current_example_desc.push_str(&code);
                    current_example_desc.push('`');
                } else if !in_code_block && !in_blockquote && !in_heading {
                    // 独立的行内代码块（tldr 有时用这种格式）
                    if !current_example_desc.is_empty() {
                        let desc = current_example_desc.trim().trim_end_matches(':').to_string();
                        examples.push(Example {
                            description: desc,
                            code: code.to_string(),
                        });
                        current_example_desc.clear();
                    }
                }
            }
            _ => {}
        }
    }

    // 如果没有描述，使用命令名
    if description.is_empty() {
        description = name.clone();
    }

    Some(Command {
        name,
        description,
        category: platform.clone(),
        platform,
        lang,
        examples,
        content: content.to_string(),
    })
}

/// 解析本地 Markdown 文件
pub fn parse_local_markdown(content: &str, filename: &str) -> Option<Command> {
    let name = filename.trim_end_matches(".md").to_string();
    parse_tldr_markdown(content, name, "zh".to_string(), "common".to_string())
}
