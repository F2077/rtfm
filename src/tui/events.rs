use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::app::{App, Focus};

/// 事件处理结果
pub enum EventResult {
  /// 继续运行
  Continue,
  /// 需要搜索
  Search,
  /// 退出程序
  Quit,
}

/// 轮询事件
pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
  if event::poll(timeout)? {
    Ok(Some(event::read()?))
  } else {
    Ok(None)
  }
}

/// 处理按键事件
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> EventResult {
  // 全局快捷键（任何焦点状态下都生效）
  match key.code {
    // Ctrl+C 或 Ctrl+Q 强制退出
    KeyCode::Char('c') | KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      return EventResult::Quit;
    }
    // Ctrl+H 切换帮助（全局可用）
    KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.show_help = !app.show_help;
      return EventResult::Continue;
    }
    // ? 切换帮助（非搜索焦点时）
    KeyCode::Char('?') if app.focus != Focus::Search => {
      app.show_help = !app.show_help;
      return EventResult::Continue;
    }
    // Ctrl+L 切换日志面板（调试模式）
    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.toggle_logs();
      return EventResult::Continue;
    }
    // 帮助模式下 Esc 关闭帮助
    KeyCode::Esc if app.show_help => {
      app.show_help = false;
      return EventResult::Continue;
    }
    _ => {}
  }

  // 帮助模式下只响应关闭
  if app.show_help {
    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('?')) {
      app.show_help = false;
    }
    return EventResult::Continue;
  }

  // 根据焦点处理事件
  match app.focus {
    Focus::Search => handle_search_input(app, key),
    Focus::List => handle_list_input(app, key),
    Focus::Detail => handle_detail_input(app, key),
  }
}

fn handle_search_input(app: &mut App, key: KeyEvent) -> EventResult {
  match key.code {
    // 清空 (Ctrl+U)
    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      app.clear_search();
      EventResult::Search
    }
    // Esc: 搜索框为空时退出，否则清空
    KeyCode::Esc => {
      if app.query.is_empty() {
        EventResult::Quit
      } else {
        app.clear_search();
        EventResult::Search
      }
    }
    // 输入字符
    KeyCode::Char(c) => {
      app.input_char(c);
      EventResult::Search
    }
    // 删除
    KeyCode::Backspace => {
      app.delete_char();
      EventResult::Search
    }
    KeyCode::Delete => {
      app.delete_char_forward();
      EventResult::Search
    }
    // 光标移动
    KeyCode::Left => {
      app.cursor_left();
      EventResult::Continue
    }
    KeyCode::Right => {
      app.cursor_right();
      EventResult::Continue
    }
    KeyCode::Home => {
      app.cursor_home();
      EventResult::Continue
    }
    KeyCode::End => {
      app.cursor_end();
      EventResult::Continue
    }
    // 切换焦点
    KeyCode::Tab | KeyCode::Down => {
      if !app.results.is_empty() {
        app.focus = Focus::List;
      }
      EventResult::Continue
    }
    KeyCode::Enter => {
      if !app.results.is_empty() {
        app.focus = Focus::List;
      }
      EventResult::Continue
    }
    _ => EventResult::Continue,
  }
}

fn handle_list_input(app: &mut App, key: KeyEvent) -> EventResult {
  match key.code {
    // 导航
    KeyCode::Up | KeyCode::Char('k') => {
      app.list_up();
      EventResult::Continue
    }
    KeyCode::Down | KeyCode::Char('j') => {
      app.list_down();
      EventResult::Continue
    }
    KeyCode::PageUp => {
      app.list_page_up();
      EventResult::Continue
    }
    KeyCode::PageDown => {
      app.list_page_down();
      EventResult::Continue
    }
    // Jump to top (Home or 'g' for vim-style gg)
    KeyCode::Home | KeyCode::Char('g') => {
      app.selected = 0;
      app.detail_scroll = 0;
      EventResult::Continue
    }
    // Jump to end (End or 'G' for vim-style)
    KeyCode::End | KeyCode::Char('G') => {
      app.selected = app.results.len().saturating_sub(1);
      app.detail_scroll = 0;
      EventResult::Continue
    }
    // 切换焦点
    KeyCode::Tab => {
      app.next_focus();
      EventResult::Continue
    }
    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
      app.focus = Focus::Detail;
      EventResult::Continue
    }
    KeyCode::Char('/') | KeyCode::Esc => {
      app.focus = Focus::Search;
      EventResult::Continue
    }
    // 在列表中也可以输入搜索
    KeyCode::Char(c) if c.is_alphanumeric() || c == ' ' => {
      app.focus = Focus::Search;
      app.input_char(c);
      EventResult::Search
    }
    _ => EventResult::Continue,
  }
}

fn handle_detail_input(app: &mut App, key: KeyEvent) -> EventResult {
  match key.code {
    // 滚动
    KeyCode::Up | KeyCode::Char('k') => {
      app.detail_scroll_up();
      EventResult::Continue
    }
    KeyCode::Down | KeyCode::Char('j') => {
      app.detail_scroll_down();
      EventResult::Continue
    }
    KeyCode::PageUp | KeyCode::Char('g') => {
      app.detail_scroll = app.detail_scroll.saturating_sub(10);
      EventResult::Continue
    }
    KeyCode::PageDown | KeyCode::Char('G') => {
      app.detail_scroll = app.detail_scroll.saturating_add(10).min(app.detail_max_scroll);
      EventResult::Continue
    }
    // Jump to top
    KeyCode::Home => {
      app.detail_scroll = 0;
      EventResult::Continue
    }
    // Jump to end
    KeyCode::End => {
      app.detail_scroll = app.detail_max_scroll;
      EventResult::Continue
    }
    // 切换焦点
    KeyCode::Tab => {
      app.next_focus();
      EventResult::Continue
    }
    KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
      app.focus = Focus::List;
      EventResult::Continue
    }
    KeyCode::Char('/') => {
      app.focus = Focus::Search;
      EventResult::Continue
    }
    _ => EventResult::Continue,
  }
}
