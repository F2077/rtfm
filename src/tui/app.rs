use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::search::{SearchEngine, SearchResult};
use crate::storage::Database;

/// 日志缓冲区（线程安全）
pub type LogBuffer = Arc<Mutex<VecDeque<String>>>;

/// 创建日志缓冲区
pub fn create_log_buffer(size: usize) -> LogBuffer {
  Arc::new(Mutex::new(VecDeque::with_capacity(size)))
}

/// 焦点位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
  Search,
  List,
  Detail,
}

/// 应用状态
pub struct App {
  /// 数据库
  pub db: Database,
  /// 搜索引擎
  pub search: Arc<RwLock<SearchEngine>>,
  /// 数据目录
  #[allow(dead_code)]
  pub data_dir: PathBuf,
  /// 应用配置
  pub config: AppConfig,

  /// 搜索查询
  pub query: String,
  /// 光标位置
  pub cursor: usize,
  /// 搜索结果
  pub results: Vec<SearchResult>,
  /// 当前选中的索引
  pub selected: usize,
  /// 详情滚动位置
  pub detail_scroll: u16,
  /// 当前焦点
  pub focus: Focus,

  /// 状态消息
  pub status: String,
  /// 是否正在加载
  pub loading: bool,
  /// 命令总数
  #[allow(dead_code)]
  pub total_commands: usize,

  /// 是否显示帮助
  pub show_help: bool,
  /// 是否退出
  pub should_quit: bool,

  /// 调试模式
  pub debug_mode: bool,
  /// 日志缓冲区
  pub log_buffer: Option<LogBuffer>,
  /// 日志滚动位置（预留）
  #[allow(dead_code)]
  pub log_scroll: u16,
  /// 是否显示日志面板
  pub show_logs: bool,
}

impl App {
  pub fn with_debug(
    db: Database,
    search: SearchEngine,
    data_dir: PathBuf,
    debug_mode: bool,
    log_buffer: Option<LogBuffer>,
    config: AppConfig,
  ) -> Self {
    let total = db.count_commands().unwrap_or(0);

    Self {
      db,
      search: Arc::new(RwLock::new(search)),
      data_dir,
      config,
      query: String::new(),
      cursor: 0,
      results: Vec::new(),
      selected: 0,
      detail_scroll: 0,
      focus: Focus::Search,
      status: format!("{} commands total", total),
      loading: false,
      total_commands: total,
      show_help: false,
      should_quit: false,
      debug_mode,
      log_buffer,
      log_scroll: 0,
      show_logs: debug_mode,
    }
  }

  /// 获取日志条目
  pub fn get_logs(&self) -> Vec<String> {
    self
      .log_buffer
      .as_ref()
      .map(|buf| buf.lock().iter().cloned().collect())
      .unwrap_or_default()
  }

  /// 切换日志面板显示
  pub fn toggle_logs(&mut self) {
    if self.debug_mode {
      self.show_logs = !self.show_logs;
    }
  }

  /// 执行搜索
  pub async fn search(&mut self) {
    if self.query.is_empty() {
      self.results.clear();
      self.selected = 0;
      self.detail_scroll = 0;
      return;
    }

    self.loading = true;
    let search = self.search.read().await;
    match search.search(&self.query, None, 100) {
      Ok(response) => {
        self.results = response.results;
        self.selected = 0;
        self.detail_scroll = 0;
        self.status = format!("Found {} results ({}ms)", response.total, response.took_ms);
      }
      Err(e) => {
        self.status = format!("Search failed: {}", e);
        self.results.clear();
      }
    }
    self.loading = false;
  }

  /// 输入字符
  pub fn input_char(&mut self, c: char) {
    self.query.insert(self.cursor, c);
    self.cursor += 1;
  }

  /// 删除字符
  pub fn delete_char(&mut self) {
    if self.cursor > 0 {
      self.cursor -= 1;
      self.query.remove(self.cursor);
    }
  }

  /// 删除光标后的字符
  pub fn delete_char_forward(&mut self) {
    if self.cursor < self.query.len() {
      self.query.remove(self.cursor);
    }
  }

  /// 光标左移
  pub fn cursor_left(&mut self) {
    if self.cursor > 0 {
      self.cursor -= 1;
    }
  }

  /// 光标右移
  pub fn cursor_right(&mut self) {
    if self.cursor < self.query.len() {
      self.cursor += 1;
    }
  }

  /// 光标移到开头
  pub fn cursor_home(&mut self) {
    self.cursor = 0;
  }

  /// 光标移到结尾
  pub fn cursor_end(&mut self) {
    self.cursor = self.query.len();
  }

  /// 清空搜索
  pub fn clear_search(&mut self) {
    self.query.clear();
    self.cursor = 0;
    self.results.clear();
    self.selected = 0;
    self.detail_scroll = 0;
  }

  /// 列表上移
  pub fn list_up(&mut self) {
    if self.selected > 0 {
      self.selected -= 1;
      self.detail_scroll = 0;
    }
  }

  /// 列表下移
  pub fn list_down(&mut self) {
    if self.selected + 1 < self.results.len() {
      self.selected += 1;
      self.detail_scroll = 0;
    }
  }

  /// 列表翻页上
  pub fn list_page_up(&mut self) {
    self.selected = self.selected.saturating_sub(10);
    self.detail_scroll = 0;
  }

  /// 列表翻页下
  pub fn list_page_down(&mut self) {
    self.selected = (self.selected + 10).min(self.results.len().saturating_sub(1));
    self.detail_scroll = 0;
  }

  /// 详情滚动上
  pub fn detail_scroll_up(&mut self) {
    self.detail_scroll = self.detail_scroll.saturating_sub(1);
  }

  /// 详情滚动下
  pub fn detail_scroll_down(&mut self) {
    self.detail_scroll = self.detail_scroll.saturating_add(1);
  }

  /// 切换焦点
  pub fn next_focus(&mut self) {
    self.focus = match self.focus {
      Focus::Search => Focus::List,
      Focus::List => Focus::Detail,
      Focus::Detail => Focus::Search,
    };
  }

  /// 获取当前选中的命令名和语言
  pub fn selected_command(&self) -> Option<(&str, &str)> {
    self.results.get(self.selected).map(|r| (r.name.as_str(), r.lang.as_str()))
  }

  /// 获取命令详情
  pub fn get_command_detail(&self, name: &str, lang: &str) -> Option<String> {
    // 优先查询指定语言，如果没有则尝试中文，再尝试英文
    let cmd = self
      .db
      .get_command(name, lang)
      .ok()
      .flatten()
      .or_else(|| self.db.get_command(name, "zh").ok().flatten())
      .or_else(|| self.db.get_command(name, "en").ok().flatten());

    cmd.map(|cmd| {
      let mut content = format!("# {}\n\n{}\n\n", cmd.name, cmd.description);
      for example in &cmd.examples {
        content.push_str(&format!("## {}\n```\n{}\n```\n\n", example.description, example.code));
      }
      content
    })
  }
}
