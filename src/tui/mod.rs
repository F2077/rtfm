pub mod app;
pub mod events;
pub mod ui;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::{
  event::{Event, KeyEventKind},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tracing::debug;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use crate::config::AppConfig;
use crate::search::SearchEngine;
use crate::storage::Database;

use app::{create_log_buffer, App, LogBuffer};
use events::{handle_key_event, poll_event, EventResult};

/// 自定义 tracing layer，将日志写入缓冲区
struct LogBufferLayer {
  buffer: LogBuffer,
  max_size: usize,
}

impl<S> Layer<S> for LogBufferLayer
where
  S: tracing::Subscriber,
{
  fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
    let mut visitor = LogVisitor::default();
    event.record(&mut visitor);

    let level = event.metadata().level();
    let target = event.metadata().target();
    let message = visitor.message.unwrap_or_default();

    // 简化 target 显示
    let short_target = target.split("::").last().unwrap_or(target);
    let log_line = format!("[{}] {} - {}", level, short_target, message);

    let mut buf = self.buffer.lock();
    if buf.len() >= self.max_size {
      buf.pop_front();
    }
    buf.push_back(log_line);
  }
}

#[derive(Default)]
struct LogVisitor {
  message: Option<String>,
}

impl tracing::field::Visit for LogVisitor {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if field.name() == "message" {
      self.message = Some(format!("{:?}", value));
    }
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if field.name() == "message" {
      self.message = Some(value.to_string());
    }
  }
}

/// 运行 TUI 界面
pub async fn run(
  db: Database,
  search: SearchEngine,
  data_dir: PathBuf,
  debug_mode: bool,
  config: AppConfig,
) -> anyhow::Result<()> {
  // 创建日志缓冲区
  let log_buffer = if debug_mode {
    Some(create_log_buffer(config.tui.log_buffer_size))
  } else {
    None
  };

  // 初始化日志系统
  init_tui_logging(&data_dir, log_buffer.clone(), debug_mode, &config);

  // 初始化终端
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // 创建应用
  let mut app = App::with_debug(db, search, data_dir, debug_mode, log_buffer, config);

  debug!("TUI started");

  // 主循环
  let result = run_app(&mut terminal, &mut app).await;

  // 恢复终端
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;

  debug!("TUI exited");

  result
}

/// 初始化 TUI 日志系统
fn init_tui_logging(
  data_dir: &Path,
  log_buffer: Option<LogBuffer>,
  debug_mode: bool,
  config: &AppConfig,
) {
  let log_dir = data_dir.join(&config.storage.log_dirname);
  std::fs::create_dir_all(&log_dir).ok();

  let file_appender = tracing_appender::rolling::daily(&log_dir, "rtfm.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

  // 保持 guard 存活
  Box::leak(Box::new(guard));

  let env_filter =
    tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| {
      if debug_mode {
        config.logging.debug_level.clone()
      } else {
        config.logging.level.clone()
      }
    }));

  let file_layer = tracing_subscriber::fmt::layer()
    .with_writer(non_blocking)
    .with_ansi(false);

  if let Some(buffer) = log_buffer {
    let buffer_layer = LogBufferLayer {
      buffer,
      max_size: config.tui.log_buffer_size,
    };
    tracing_subscriber::registry()
      .with(env_filter)
      .with(file_layer)
      .with(buffer_layer)
      .init();
  } else {
    tracing_subscriber::registry()
      .with(env_filter)
      .with(file_layer)
      .init();
  }
}

async fn run_app(
  terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
  app: &mut App,
) -> anyhow::Result<()> {
  let poll_timeout = Duration::from_millis(app.config.tui.poll_timeout_ms);

  loop {
    // 渲染
    terminal.draw(|f| ui::render(f, app))?;

    // 处理事件
    if let Some(event) = poll_event(poll_timeout)? {
      match event {
        Event::Key(key) => {
          // 只处理按下事件，忽略释放事件（修复 Windows 上字符重复问题）
          if key.kind != KeyEventKind::Press {
            continue;
          }

          debug!("Key: {:?}", key.code);

          match handle_key_event(app, key) {
            EventResult::Continue => {}
            EventResult::Search => {
              app.search().await;
            }
            EventResult::Quit => {
              break;
            }
          }
        }
        Event::Resize(w, h) => {
          debug!("Resize: {}x{}", w, h);
        }
        _ => {}
      }
    }

    if app.should_quit {
      break;
    }
  }

  Ok(())
}
