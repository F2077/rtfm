mod api;
mod cli;
mod config;
mod learn;
mod search;
mod storage;
mod tui;
mod update;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use clap::Parser;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};
use config::AppConfig;
use search::SearchEngine;
use storage::Database;

pub struct AppState {
  pub db: Database,
  pub search: RwLock<SearchEngine>,
  pub data_dir: PathBuf,
  pub config: AppConfig,
}

fn get_data_dir(config: &AppConfig) -> PathBuf {
  config.get_data_dir()
}

/// 初始化终端日志（用于 CLI 命令）
fn init_console_logging(config: &AppConfig) {
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(tracing_subscriber::EnvFilter::new(
      std::env::var("RUST_LOG").unwrap_or_else(|_| config.logging.level.clone()),
    ))
    .init();
}

/// 初始化服务器日志（输出到文件）
fn init_server_logging(log_dir: &std::path::Path, config: &AppConfig, debug: bool) {
  let file_appender = tracing_appender::rolling::daily(log_dir, "rtfm.log");
  let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);
  
  // Keep guard alive
  Box::leak(Box::new(guard));

  let env_filter = tracing_subscriber::EnvFilter::new(
    std::env::var("RUST_LOG").unwrap_or_else(|_| config.logging.level.clone()),
  );

  if debug {
    // Debug mode: dual-write to file and console
    tracing_subscriber::registry()
      .with(
        tracing_subscriber::fmt::layer()
          .with_writer(non_blocking_file)
          .with_ansi(false)
      )
      .with(
        tracing_subscriber::fmt::layer()
          .with_writer(std::io::stdout)
      )
      .with(env_filter)
      .init();
  } else {
    // Normal mode: file only
    tracing_subscriber::registry()
      .with(
        tracing_subscriber::fmt::layer()
          .with_writer(non_blocking_file)
          .with_ansi(false)
      )
      .with(env_filter)
      .init();
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  
  // 加载配置
  let config = AppConfig::load_default();

  match cli.command {
    // 启动 HTTP 服务模式
    Some(Commands::Serve { port, bind, detach, debug }) => {
      if detach {
        run_server_detached(&bind, port, &config)
      } else {
        run_server(&bind, port, debug, config).await
      }
    }

    // 更新命令
    Some(Commands::Update { force }) => {
      init_console_logging(&config);
      run_update(force, &config).await
    }

    // 导入命令
    Some(Commands::Import { path }) => {
      init_console_logging(&config);
      run_import(&path, &config).await
    }

    // 从 --help 或 man 学习命令
    Some(Commands::Learn { command, force, man }) => {
      run_learn(&command, force, man, &config).await
    }

    // 批量学习系统 man 页面
    Some(Commands::LearnAll { section, limit, skip_existing, prefix, source }) => {
      run_learn_all(&section, limit, skip_existing, prefix.as_deref(), &source, &config).await
    }

    // 备份应用数据
    Some(Commands::Backup { output }) => {
      run_backup(&output, &config).await
    }

    // 从备份恢复数据
    Some(Commands::Restore { path, merge }) => {
      run_restore(&path, merge, &config).await
    }

    // 重置所有数据
    Some(Commands::Reset { yes }) => {
      run_reset(yes, &config).await
    }

    // 无子命令时
    None => {
      // 如果有查询参数，直接输出命令信息
      if let Some(query) = cli.query {
        run_query(&query, &cli.lang, &config).await
      } else {
        // 否则启动 TUI
        run_tui(cli.debug, config).await
      }
    }
  }
}

/// 运行 TUI 界面
async fn run_tui(debug_mode: bool, config: AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(&config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let search = SearchEngine::open(&index_path)?;

  // 启动 TUI（日志初始化在 tui::run 内部）
  tui::run(db, search, data_dir, debug_mode, config).await
}

/// 运行 HTTP 服务
async fn run_server(bind: &str, port: u16, debug: bool, config: AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(&config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化日志
  let log_dir = data_dir.join(&config.storage.log_dirname);
  std::fs::create_dir_all(&log_dir)?;

  init_server_logging(&log_dir, &config, debug);

  tracing::info!("Data directory: {:?}", data_dir);

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;
  tracing::info!("Database opened: {:?}", db_path);

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let search = SearchEngine::open(&index_path)?;
  tracing::info!("Search index opened: {:?}", index_path);

  // 创建应用状态
  let max_upload_size = config.server.max_upload_size;
  let state = Arc::new(AppState {
    db,
    search: RwLock::new(search),
    data_dir: data_dir.clone(),
    config,
  });

  // 配置 CORS
  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  // 构建路由
  let app = Router::new()
    .merge(api::routes_with_docs(max_upload_size))
    .layer(cors)
    .with_state(state);

  // 启动服务器
  let addr: SocketAddr = format!("{}:{}", bind, port).parse()?;
  println!("RTFM HTTP server listening on http://{}", addr);
  println!("Swagger UI: http://{}/swagger-ui", addr);
  println!("Logs: {}", log_dir.display());
  if debug {
    println!("Debug mode: ON (logs also printed to console)");
  }
  println!("Press Ctrl+C to stop");
  tracing::info!("HTTP server listening on http://{}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await?;

  // Graceful shutdown with Ctrl+C
  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;

  println!("\nServer stopped gracefully");
  tracing::info!("Server stopped");

  Ok(())
}

/// Wait for Ctrl+C signal
async fn shutdown_signal() {
  tokio::signal::ctrl_c()
    .await
    .expect("Failed to install Ctrl+C handler");
}

/// Run server in detached/background mode
fn run_server_detached(bind: &str, port: u16, config: &AppConfig) -> anyhow::Result<()> {
  use std::process::{Command, Stdio};

  let exe = std::env::current_exe()?;
  let log_dir = get_data_dir(config).join(&config.storage.log_dirname);
  std::fs::create_dir_all(&log_dir)?;

  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const DETACHED_PROCESS: u32 = 0x00000008;

    Command::new(&exe)
      .args(["serve", "--port", &port.to_string(), "--bind", bind])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
      .spawn()?;
  }

  #[cfg(unix)]
  {
    Command::new(&exe)
      .args(["serve", "--port", &port.to_string(), "--bind", bind])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()?;
  }

  println!("RTFM server started in background");
  println!("  Address: http://{}:{}", bind, port);
  println!("  Swagger: http://{}:{}/swagger-ui", bind, port);
  println!("  Logs: {}", log_dir.display());
  println!("\nTo stop: kill the rtfm process or use task manager");

  Ok(())
}

/// 运行更新命令
async fn run_update(force: bool, config: &AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let mut search = SearchEngine::open(&index_path)?;

  // 检查更新
  println!("Checking for updates...");
  let update_info = update::check_github_release(&config.update).await?;

  if !force {
    let current = db.get_metadata()?.map(|m| m.version).unwrap_or_default();
    if current == update_info.tag_name {
      println!("Already up to date: {}", current);
      return Ok(());
    }
  }

  println!("New version found: {}", update_info.tag_name);

  // 下载
  let Some(url) = update_info.download_url else {
    anyhow::bail!("Download URL not found");
  };

  println!("Downloading: {}", url);
  let client = reqwest::Client::new();
  let response = client.get(&url).send().await?;
  let bytes = response.bytes().await?;

  // 解析
  println!("Parsing cheatsheets...");
  let commands = update::parse_tldr_archive(&bytes)?;
  println!("Parsed {} commands", commands.len());

  // 保存
  println!("Saving to database...");
  db.clear_commands()?;
  db.save_commands(&commands)?;

  // 重建索引
  println!("Rebuilding search index...");
  search.index_commands(&commands)?;

  // 更新元数据
  let metadata = storage::Metadata {
    version: update_info.tag_name.clone(),
    command_count: commands.len(),
    last_update: chrono::Utc::now().to_rfc3339(),
    languages: vec!["zh".to_string(), "en".to_string()],
  };
  db.save_metadata(&metadata)?;

  println!("Update complete! Version: {}", update_info.tag_name);
  Ok(())
}

/// 运行导入命令
async fn run_import(path: &str, config: &AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let mut search = SearchEngine::open(&index_path)?;

  let path = PathBuf::from(path);
  if !path.exists() {
    anyhow::bail!("Path does not exist: {:?}", path);
  }

  let (commands, _total_files, skipped) = import_from_path(&path)?;

  if commands.is_empty() {
    println!("No valid Markdown files found.");
    println!();
    println!("Files must follow the tldr-pages format:");
    println!("  # command-name");
    println!("  > Brief description.");
    println!("  - Example description:");
    println!("  `command --option {{{{arg}}}}`");
    println!();
    println!("See: https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md");
    return Ok(());
  }

  println!("Importing {} commands...", commands.len());
  if skipped > 0 {
    println!("  (skipped {} files without valid tldr format)", skipped);
  }

  db.save_commands(&commands)?;
  search.index_commands(&commands)?;

  println!("Import complete! {} commands imported.", commands.len());
  Ok(())
}

/// Import commands from a path (file, directory, or archive)
/// Returns (commands, total_files_scanned, skipped_count)
fn import_from_path(path: &PathBuf) -> anyhow::Result<(Vec<storage::Command>, usize, usize)> {
  use std::io::Read;

  let mut commands = Vec::new();
  let mut total_files = 0;
  let mut skipped = 0;

  if path.is_dir() {
    // Directory of markdown files
    for entry in walkdir(path)? {
      if entry.extension().map(|e| e == "md").unwrap_or(false) {
        total_files += 1;
        let content = std::fs::read_to_string(&entry)?;
        let filename = entry.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        if let Some(cmd) = update::parse_local_markdown(&content, filename) {
          commands.push(cmd);
        } else {
          skipped += 1;
        }
      }
    }
  } else if path.is_file() {
    // Detect file type by extension
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    match ext.to_lowercase().as_str() {
      "md" => {
        // Single markdown file
        total_files += 1;
        let content = std::fs::read_to_string(path)?;
        if let Some(cmd) = update::parse_local_markdown(&content, filename) {
          commands.push(cmd);
        } else {
          skipped += 1;
        }
      }
      "zip" => {
        // ZIP archive
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        for i in 0..archive.len() {
          let mut entry = archive.by_index(i)?;
          let name = entry.name().to_string();
          if name.ends_with(".md") && !entry.is_dir() {
            total_files += 1;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            let md_name = std::path::Path::new(&name)
              .file_name()
              .and_then(|n| n.to_str())
              .unwrap_or("unknown");
            if let Some(cmd) = update::parse_local_markdown(&content, md_name) {
              commands.push(cmd);
            } else {
              skipped += 1;
            }
          }
        }
      }
      "gz" | "tgz" => {
        // tar.gz or .tgz archive
        let file = std::fs::File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        for entry in archive.entries()? {
          let mut entry = entry?;
          let path = entry.path()?.to_path_buf();
          if path.extension().map(|e| e == "md").unwrap_or(false) {
            total_files += 1;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            let md_name = path
              .file_name()
              .and_then(|n| n.to_str())
              .unwrap_or("unknown");
            if let Some(cmd) = update::parse_local_markdown(&content, md_name) {
              commands.push(cmd);
            } else {
              skipped += 1;
            }
          }
        }
      }
      "tar" => {
        // Plain tar archive
        let file = std::fs::File::open(path)?;
        let mut archive = tar::Archive::new(file);
        for entry in archive.entries()? {
          let mut entry = entry?;
          let path = entry.path()?.to_path_buf();
          if path.extension().map(|e| e == "md").unwrap_or(false) {
            total_files += 1;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            let md_name = path
              .file_name()
              .and_then(|n| n.to_str())
              .unwrap_or("unknown");
            if let Some(cmd) = update::parse_local_markdown(&content, md_name) {
              commands.push(cmd);
            } else {
              skipped += 1;
            }
          }
        }
      }
      _ => {
        // Try to read as markdown anyway
        total_files += 1;
        if let Ok(content) = std::fs::read_to_string(path) {
          if let Some(cmd) = update::parse_local_markdown(&content, filename) {
            commands.push(cmd);
          } else {
            skipped += 1;
          }
        } else {
          skipped += 1;
        }
      }
    }
  }

  Ok((commands, total_files, skipped))
}

/// 简单的目录遍历
fn walkdir(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
  let mut files = Vec::new();
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      files.extend(walkdir(&path)?);
    } else {
      files.push(path);
    }
  }
  Ok(files)
}

/// 直接查询命令并输出到终端
async fn run_query(query: &str, lang: &str, config: &AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  if !db_path.exists() {
    eprintln!("Database not found. Run 'rtfm update' first.");
    std::process::exit(1);
  }

  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let search = SearchEngine::open(&index_path)?;

  // 尝试多种匹配方式
  // 1. 精确匹配命令名
  let name = query.trim();
  let cmd = db
    .get_command(name, lang)
    .ok()
    .flatten()
    .or_else(|| db.get_command(name, "en").ok().flatten())
    .or_else(|| db.get_command(name, "zh").ok().flatten());

  if let Some(cmd) = cmd {
    print_command(&cmd);
    return Ok(());
  }

  // 2. 尝试把空格替换成 `-`（tldr 命名规范）
  let normalized = name.replace(' ', "-");
  if normalized != name {
    let cmd = db
      .get_command(&normalized, lang)
      .ok()
      .flatten()
      .or_else(|| db.get_command(&normalized, "en").ok().flatten())
      .or_else(|| db.get_command(&normalized, "zh").ok().flatten());

    if let Some(cmd) = cmd {
      print_command(&cmd);
      return Ok(());
    }
  }

  // 3. 全文检索
  let results = search.search(query, None, 10)?;

  if results.results.is_empty() {
    eprintln!("No results for '{}'.", query);
    eprintln!("Try 'rtfm update' to download the latest cheatsheets.");
    std::process::exit(1);
  }

  // 如果只有一个结果，直接显示
  if results.results.len() == 1 {
    let r = &results.results[0];
    if let Some(cmd) = db
      .get_command(&r.name, &r.lang)
      .ok()
      .flatten()
    {
      print_command(&cmd);
      return Ok(());
    }
  }

  // 多个结果，列出供选择
  println!("\x1b[1mFound {} results for '{}':\x1b[0m\n", results.results.len(), query);
  for (i, r) in results.results.iter().enumerate() {
    println!(
      "  \x1b[32m{:2}.\x1b[0m \x1b[1m{}\x1b[0m \x1b[90m[{}]\x1b[0m",
      i + 1,
      r.name,
      r.lang
    );
    println!("      \x1b[90m{}\x1b[0m", truncate(&r.description, 60));
  }
  println!();
  println!("Use \x1b[36mrtfm <command>\x1b[0m to view details.");

  Ok(())
}

/// 截断字符串
fn truncate(s: &str, max_len: usize) -> String {
  if s.chars().count() <= max_len {
    s.to_string()
  } else {
    let truncated: String = s.chars().take(max_len - 3).collect();
    format!("{}...", truncated)
  }
}

/// 格式化输出命令信息
fn print_command(cmd: &storage::Command) {
  // 命令名（绿色粗体）
  println!("\x1b[1;32m{}\x1b[0m", cmd.name);
  println!();

  // 描述
  println!("{}", cmd.description);
  println!();

  // 示例
  for example in &cmd.examples {
    // 示例描述（黄色）
    println!("\x1b[33m- {}\x1b[0m", example.description);
    // 代码（青色）
    println!("  \x1b[36m{}\x1b[0m", example.code);
    println!();
  }
}

/// 从 --help 或 man 学习命令
async fn run_learn(command: &str, force: bool, prefer_man: bool, config: &AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let mut search = SearchEngine::open(&index_path)?;

  // 检查是否已存在
  if !force {
    if let Ok(Some(_)) = db.get_command(command, "local") {
      println!("Command '{}' already learned. Use --force to re-learn.", command);
      return Ok(());
    }
  }

  println!("Learning '{}'...", command);

  // 获取帮助内容，根据优先级尝试
  let (content, source) = if prefer_man {
    // 优先 man
    match learn::get_man_page(command) {
      Ok(result) => result,
      Err(man_e) => {
        // man 失败，尝试 --help
        match learn::get_help_output(command) {
          Ok(result) => result,
          Err(help_e) => {
            // 两个都失败
            print_learn_error(command, &help_e, &man_e);
            return Ok(());
          }
        }
      }
    }
  } else {
    // 优先 --help
    match learn::get_help_output(command) {
      Ok(result) => result,
      Err(help_e) => {
        // --help 失败，尝试 man
        match learn::get_man_page(command) {
          Ok(result) => result,
          Err(man_e) => {
            // 两个都失败
            print_learn_error(command, &help_e, &man_e);
            return Ok(());
          }
        }
      }
    }
  };

  println!("Got {} bytes from {}", content.len(), source);

  // 解析帮助内容
  let cmd = learn::parse_help_content(command, &content, &source);

  // 保存到数据库
  db.save_command(&cmd)?;
  println!("Saved to database");

  // 更新索引（增量）
  search.index_single_command(&cmd)?;
  println!("Indexed for search");

  println!("\n\x1b[32mLearned '{}' successfully!\x1b[0m", command);
  println!("Try: rtfm {}", command);

  Ok(())
}

/// 打印学习命令失败的人性化错误信息
fn print_learn_error(command: &str, help_err: &anyhow::Error, man_err: &anyhow::Error) {
  let help_err_str = help_err.to_string();
  let man_err_str = man_err.to_string();

  eprintln!("\n\x1b[1;31mFailed to learn '{}'.\x1b[0m\n", command);

  // 分析错误类型
  // 命令不存在：--help 返回 "program not found" 或类似错误
  let cmd_not_found = help_err_str.contains("program not found") 
    || help_err_str.contains("No such file")
    || help_err_str.contains("not recognized")
    || help_err_str.contains("command not found");
  
  // man 不可用（Windows 常见情况）
  let man_not_available = man_err_str.contains("program not found");

  if cmd_not_found && man_not_available {
    // 命令不存在，且 man 也不可用
    eprintln!("\x1b[33mCommand '{}' not found on this system.\x1b[0m", command);
    eprintln!();
    eprintln!("Possible reasons:");
    eprintln!("  - The command is not installed");
    eprintln!("  - The command is not in your PATH");
    eprintln!("  - The command name is misspelled");
    eprintln!();
    eprintln!("Try:");
    eprintln!("  - Install the command first, then run: rtfm learn {}", command);
    eprintln!("  - Use 'rtfm update' to download cheatsheets from tldr-pages");
  } else if cmd_not_found {
    // 命令不存在，但 man 可用（返回了其他错误）
    eprintln!("\x1b[33mCommand '{}' not found on this system.\x1b[0m", command);
    eprintln!();
    eprintln!("The command is not installed or not in PATH.");
    eprintln!();
    eprintln!("Try:");
    eprintln!("  - Install the command first, then run: rtfm learn {}", command);
    eprintln!("  - Use 'rtfm update' to download cheatsheets from tldr-pages");
    eprintln!("  - Use 'rtfm learn {} --man' to check if man page exists", command);
  } else if man_not_available {
    // 命令可能存在但 --help 失败了，且 man 不可用
    eprintln!("\x1b[33mCould not get help for '{}', and 'man' is not available.\x1b[0m", command);
    eprintln!();

    #[cfg(target_os = "windows")]
    {
      eprintln!("On Windows, 'man' pages are not available by default.");
      eprintln!("The command '{}' exists but --help didn't provide usable output.", command);
      eprintln!();
      eprintln!("Details:");
      eprintln!("  --help: {}", help_err_str);
      eprintln!();
      eprintln!("Alternatives:");
      eprintln!("  - Use 'rtfm update' to download cheatsheets from tldr-pages");
      eprintln!("  - Check if '{}' supports a different help flag (e.g., /?, -h)", command);
    }

    #[cfg(not(target_os = "windows"))]
    {
      eprintln!("Details:");
      eprintln!("  --help: {}", help_err_str);
      eprintln!();
      eprintln!("Try:");
      eprintln!("  - Install man-db package (e.g., apt install man-db)");
      eprintln!("  - Use 'rtfm update' to download cheatsheets from tldr-pages");
    }
  } else {
    // 其他错误，显示原始信息
    eprintln!("The command exists but didn't provide usable help output.");
    eprintln!();
    eprintln!("Details:");
    eprintln!("  --help: {}", help_err_str);
    eprintln!("  man:    {}", man_err_str);
    eprintln!();
    eprintln!("Try:");
    eprintln!("  - Run '{} --help' manually to check the output", command);
    eprintln!("  - Use 'rtfm update' to download cheatsheets from tldr-pages");
  }

  eprintln!();
}

/// 批量学习命令（跨平台）
/// - Linux/macOS: 默认从 man 页面学习
/// - Windows: 默认从 PowerShell cmdlet 学习
/// - 所有平台: 可以从 PATH 中的可执行文件学习
async fn run_learn_all(
  section: &str,
  limit: usize,
  skip_existing: bool,
  prefix: Option<&str>,
  source: &str,
  config: &AppConfig,
) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);
  std::fs::create_dir_all(&data_dir)?;

  // 初始化数据库
  let db_path = data_dir.join(&config.storage.db_filename);
  let db = Database::open(&db_path)?;

  // 初始化搜索引擎
  let index_path = data_dir.join(&config.storage.index_dirname);
  let mut search = SearchEngine::open(&index_path)?;

  // 确定实际使用的来源
  let actual_source = if source == "auto" {
    #[cfg(target_os = "windows")]
    { "powershell" }
    #[cfg(not(target_os = "windows"))]
    { "man" }
  } else {
    source
  };

  println!("Source: {}", actual_source);

  // 获取命令列表
  let commands = match actual_source {
    "man" => {
      println!("Listing man pages in section {}...", section);
      learn::list_man_pages(section)?
    }
    "powershell" | "path" => {
      learn::list_available_commands(actual_source)?
    }
    _ => {
      anyhow::bail!("Unknown source '{}'. Use 'man', 'powershell', 'path', or 'auto'.", source);
    }
  };

  if commands.is_empty() {
    println!("No commands found.");
    print_learn_all_help(actual_source);
    return Ok(());
  }

  println!("Found {} commands", commands.len());

  // 过滤
  let commands: Vec<_> = commands
    .into_iter()
    .filter(|(name, _)| {
      if let Some(p) = prefix {
        name.to_lowercase().starts_with(&p.to_lowercase())
      } else {
        true
      }
    })
    .collect();

  if let Some(p) = prefix {
    println!("Filtered to {} commands with prefix '{}'", commands.len(), p);
  }

  // 限制数量
  let commands: Vec<_> = if limit > 0 && commands.len() > limit {
    println!("Limiting to {} commands", limit);
    commands.into_iter().take(limit).collect()
  } else {
    commands
  };

  let total = commands.len();
  let mut learned = 0;
  let mut skipped = 0;
  let mut failed = 0;

  for (i, (name, _desc)) in commands.iter().enumerate() {
    // 跳过已存在的
    if skip_existing {
      if let Ok(Some(_)) = db.get_command(name, "local") {
        skipped += 1;
        continue;
      }
    }

    print!("\r[{}/{}] Learning '{}'...", i + 1, total, name);
    std::io::Write::flush(&mut std::io::stdout())?;

    // 根据来源类型获取帮助
    let result = match actual_source {
      "man" => learn::get_man_page_with_section(name, section),
      _ => learn::get_help_output(name),
    };

    match result {
      Ok((content, src)) => {
        let cmd = learn::parse_help_content(name, &content, &src);
        if db.save_command(&cmd).is_ok()
          && search.index_single_command(&cmd).is_ok()
        {
          learned += 1;
        }
      }
      Err(_) => {
        failed += 1;
      }
    }
  }

  println!("\r\x1b[K"); // 清除进度行
  println!("\n\x1b[32mDone!\x1b[0m");
  println!("  Learned: {}", learned);
  if skipped > 0 {
    println!("  Skipped: {} (already exist)", skipped);
  }
  if failed > 0 {
    println!("  Failed:  {}", failed);
  }
  println!("\nTotal commands in database: {}", db.count_commands()?);

  Ok(())
}

/// 打印 learn-all 帮助信息
fn print_learn_all_help(source: &str) {
  match source {
    "man" => {
      println!("\nMan sections:");
      println!("  1 - User commands");
      println!("  2 - System calls");
      println!("  3 - Library functions");
      println!("  4 - Special files");
      println!("  5 - File formats");
      println!("  6 - Games");
      println!("  7 - Miscellaneous");
      println!("  8 - System administration");
    }
    "powershell" => {
      println!("\nTry:");
      println!("  rtfm learn-all --source path  # Learn from PATH executables");
      println!("  rtfm learn cargo              # Learn a specific command");
    }
    _ => {
      println!("\nAvailable sources:");
      #[cfg(target_os = "windows")]
      {
        println!("  --source powershell  # PowerShell cmdlets (default on Windows)");
        println!("  --source path        # Executables in PATH");
      }
      #[cfg(not(target_os = "windows"))]
      {
        println!("  --source man         # Man pages (default on Linux/macOS)");
        println!("  --source path        # Executables in PATH");
      }
    }
  }
}

/// 备份应用数据到归档文件
async fn run_backup(output: &str, config: &AppConfig) -> anyhow::Result<()> {
  use flate2::write::GzEncoder;
  use flate2::Compression;
  use tar::Builder;

  let data_dir = get_data_dir(config);

  // 检查数据目录
  let db_path = data_dir.join(&config.storage.db_filename);
  if !db_path.exists() {
    anyhow::bail!("No data found. Run 'rtfm update' or 'rtfm learn' first.");
  }

  println!("Backing up data from {:?}...", data_dir);

  // 创建 tar.gz 文件
  let output_path = PathBuf::from(output);
  let file = std::fs::File::create(&output_path)?;
  let enc = GzEncoder::new(file, Compression::default());
  let mut tar = Builder::new(enc);

  // 添加数据库文件
  if db_path.exists() {
    println!("  Adding {}...", config.storage.db_filename);
    tar.append_path_with_name(&db_path, &config.storage.db_filename)?;
  }

  // 添加索引目录
  let index_path = data_dir.join(&config.storage.index_dirname);
  if index_path.exists() {
    println!("  Adding {}/...", config.storage.index_dirname);
    tar.append_dir_all(&config.storage.index_dirname, &index_path)?;
  }

  // 添加配置文件（从数据目录）
  let config_path = data_dir.join("config.toml");
  if config_path.exists() {
    println!("  Adding config.toml...");
    tar.append_path_with_name(&config_path, "config.toml")?;
  } else {
    // 如果数据目录没有配置文件，导出当前配置
    let config_content = config.to_toml();
    let config_bytes = config_content.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path("config.toml")?;
    header.set_size(config_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, config_bytes)?;
    println!("  Adding config.toml (current config)...");
  }

  // 创建 README
  let readme = create_backup_readme();
  let readme_bytes = readme.as_bytes();
  let mut header = tar::Header::new_gnu();
  header.set_path("README.md")?;
  header.set_size(readme_bytes.len() as u64);
  header.set_mode(0o644);
  header.set_cksum();
  tar.append(&header, readme_bytes)?;

  // 添加元数据文件
  let db = Database::open(&db_path)?;
  if let Ok(Some(meta)) = db.get_metadata() {
    let meta_json = serde_json::to_string_pretty(&meta)?;
    let meta_bytes = meta_json.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path("metadata.json")?;
    header.set_size(meta_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, meta_bytes)?;
    println!("  Adding metadata.json...");
  }

  tar.finish()?;

  let file_size = std::fs::metadata(&output_path)?.len();
  println!("\n\x1b[32mBackup complete!\x1b[0m");
  println!("  Output: {}", output_path.display());
  println!("  Size:   {} bytes ({:.2} MB)", file_size, file_size as f64 / 1024.0 / 1024.0);
  println!("\nTo restore on another machine:");
  println!("  rtfm restore {}", output);

  Ok(())
}

/// 创建备份 README
fn create_backup_readme() -> String {
  format!(r#"# RTFM Backup

This archive contains backup data from RTFM (Read The F***ing Manual).

## Contents

- `data.redb` - Command database (redb format)
- `index/` - Full-text search index (Tantivy format)
- `config.toml` - Application configuration
- `metadata.json` - Backup metadata (version, command count, etc.)

## Restore

To restore this backup on another machine:

```bash
rtfm restore rtfm-backup.tar.gz
```

Options:
- `--merge` - Merge with existing data instead of replacing

## Version Info

- Backup date: {}
- RTFM version: {}

## Data Format

The database uses redb (Rust embedded database) format.
The search index uses Tantivy format.

These files are cross-platform compatible (Windows/Linux/macOS).
"#, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"), env!("CARGO_PKG_VERSION"))
}

/// 从备份恢复数据
async fn run_restore(path: &str, merge: bool, config: &AppConfig) -> anyhow::Result<()> {
  use flate2::read::GzDecoder;
  use tar::Archive;

  let archive_path = PathBuf::from(path);
  if !archive_path.exists() {
    anyhow::bail!("Backup archive not found: {}", path);
  }

  println!("Restoring from {}...", path);

  let data_dir = get_data_dir(config);
  std::fs::create_dir_all(&data_dir)?;

  // 打开归档
  let file = std::fs::File::open(&archive_path)?;
  let dec = GzDecoder::new(file);
  let mut archive = Archive::new(dec);

  // 如果不是 merge 模式，先备份并清空
  let db_path = data_dir.join(&config.storage.db_filename);
  let index_path = data_dir.join(&config.storage.index_dirname);

  if !merge && db_path.exists() {
    let backup_path = data_dir.join(format!("{}.backup", config.storage.db_filename));
    println!("  Backing up existing database to {:?}", backup_path);
    std::fs::rename(&db_path, &backup_path)?;
  }

  if !merge && index_path.exists() {
    let backup_path = data_dir.join(format!("{}.backup", config.storage.index_dirname));
    println!("  Backing up existing index to {:?}", backup_path);
    if backup_path.exists() {
      std::fs::remove_dir_all(&backup_path)?;
    }
    std::fs::rename(&index_path, &backup_path)?;
  }

  // 解压到数据目录
  println!("  Extracting files...");
  for entry in archive.entries()? {
    let mut entry = entry?;
    let path = entry.path()?;
    let path_str = path.to_string_lossy();

    // 跳过 README 和 metadata（仅用于信息）
    if path_str == "README.md" || path_str == "metadata.json" {
      continue;
    }

    let dest = data_dir.join(&*path);
    println!("    {}", path.display());

    if entry.header().entry_type().is_dir() {
      std::fs::create_dir_all(&dest)?;
    } else {
      if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
      }
      entry.unpack(&dest)?;
    }
  }

  // 验证恢复
  let db = Database::open(&db_path)?;
  let count = db.count_commands()?;

  // 检查配置文件是否恢复
  let config_path = data_dir.join("config.toml");
  let config_restored = config_path.exists();

  println!("\n\x1b[32mRestore complete!\x1b[0m");
  println!("  Commands: {}", count);
  if config_restored {
    println!("  Config:   restored to {:?}", config_path);
  }
  println!("\nTry: rtfm tar");

  Ok(())
}

/// 重置所有数据（恢复出厂设置）
async fn run_reset(skip_confirm: bool, config: &AppConfig) -> anyhow::Result<()> {
  let data_dir = get_data_dir(config);

  // 检查数据目录是否存在
  if !data_dir.exists() {
    println!("No data directory found. Nothing to reset.");
    return Ok(());
  }

  let db_path = data_dir.join(&config.storage.db_filename);
  let index_path = data_dir.join(&config.storage.index_dirname);
  let config_path = data_dir.join("config.toml");

  // 检查是否有数据
  let has_db = db_path.exists();
  let has_index = index_path.exists();
  let has_config = config_path.exists();

  if !has_db && !has_index && !has_config {
    println!("No data found. Nothing to reset.");
    return Ok(());
  }

  // 显示将要删除的内容
  println!("\x1b[1;33mWarning: This will delete all RTFM data!\x1b[0m\n");
  println!("Data directory: {:?}", data_dir);
  println!("\nThe following will be deleted:");
  if has_db {
    println!("  - {} (command database)", config.storage.db_filename);
  }
  if has_index {
    println!("  - {}/ (search index)", config.storage.index_dirname);
  }
  if has_config {
    println!("  - config.toml (local configuration)");
  }

  // 确认
  if !skip_confirm {
    println!("\n\x1b[1mAre you sure you want to continue? [y/N]\x1b[0m ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input != "y" && input != "yes" {
      println!("Aborted.");
      return Ok(());
    }
  }

  println!("\nResetting...");

  // 删除数据库
  if has_db {
    std::fs::remove_file(&db_path)?;
    println!("  Deleted {}", config.storage.db_filename);
  }

  // 删除索引目录
  if has_index {
    std::fs::remove_dir_all(&index_path)?;
    println!("  Deleted {}/", config.storage.index_dirname);
  }

  // 删除配置文件
  if has_config {
    std::fs::remove_file(&config_path)?;
    println!("  Deleted config.toml");
  }

  println!("\n\x1b[32mReset complete!\x1b[0m");
  println!("All data has been deleted. RTFM is now in factory state.");
  println!("\nTo start fresh, run:");
  println!("  rtfm update    # Download cheatsheets from tldr");
  println!("  rtfm learn ls  # Learn from local commands");

  Ok(())
}
