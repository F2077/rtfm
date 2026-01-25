use ratatui::{
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
  Frame,
};

use super::app::{App, Focus};

/// ASCII Art Logo - 翻开的手册书本造型，致敬经典 RTFM 梗
/// 固定 7 行高度，左对齐显示以保持排版
const LOGO: [&str; 7] = [
  r#"      _________                  "When all else fails...""#,
  r#"     /        /|"#,
  r#"    /  RTFM  / |                   READ THE F***ING MANUAL"#,
  r#"   /________/  |   < Go on, I DARE you to ask again!"#,
  r#"   |  ~~~~  |  |"#,
  r#"   | MANUAL |  /                   Rust-powered CLI Cheatsheet"#,
  r#"   |________|/"#,
];

/// Logo 固定高度
const LOGO_HEIGHT: u16 = 7;

/// 主界面渲染
pub fn render(frame: &mut Frame, app: &App) {
  let area = frame.area();
  // 最小高度需求：搜索框 3 + 主内容 5 + 状态栏 1 + logo 7 = 16
  // 带日志面板：16 + 10 = 26
  let min_height_for_logo = if app.show_logs { 26 } else { 16 };
  let show_logo = area.height >= min_height_for_logo;
  
  // 构建布局约束
  let constraints = if show_logo {
    if app.show_logs {
      vec![
        Constraint::Length(LOGO_HEIGHT), // Logo 区
        Constraint::Length(3),           // 搜索框
        Constraint::Min(5),              // 主内容区
        Constraint::Length(10),          // 日志面板
        Constraint::Length(1),           // 状态栏
      ]
    } else {
      vec![
        Constraint::Length(LOGO_HEIGHT), // Logo 区
        Constraint::Length(3),           // 搜索框
        Constraint::Min(5),              // 主内容区
        Constraint::Length(1),           // 状态栏
      ]
    }
  } else {
    if app.show_logs {
      vec![
        Constraint::Length(3),  // 搜索框
        Constraint::Min(5),     // 主内容区
        Constraint::Length(10), // 日志面板
        Constraint::Length(1),  // 状态栏
      ]
    } else {
      vec![
        Constraint::Length(3), // 搜索框
        Constraint::Min(5),    // 主内容区
        Constraint::Length(1), // 状态栏
      ]
    }
  };

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints)
    .split(area);

  let mut idx = 0;
  
  if show_logo {
    render_logo(frame, chunks[idx]);
    idx += 1;
  }
  
  render_search_bar(frame, app, chunks[idx]);
  idx += 1;
  
  render_main(frame, app, chunks[idx]);
  idx += 1;

  if app.show_logs {
    render_log_panel(frame, app, chunks[idx]);
    idx += 1;
  }
  
  render_status_bar(frame, app, chunks[idx]);

  // 帮助弹窗
  if app.show_help {
    render_help_popup(frame);
  }
}

/// 渲染 ASCII Art Logo（固定大小，左对齐）
fn render_logo(frame: &mut Frame, area: Rect) {
  let lines: Vec<Line> = LOGO
    .iter()
    .map(|line| {
      Line::from(Span::styled(
        *line,
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
      ))
    })
    .collect();

  let logo = Paragraph::new(lines);
  frame.render_widget(logo, area);
}

/// 渲染搜索框
fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Min(20), Constraint::Length(30)])
    .split(area);

  // 搜索框
  let search_style = if app.focus == Focus::Search {
    Style::default().fg(Color::Yellow)
  } else {
    Style::default().fg(Color::Gray)
  };

  let search_block = Block::default()
    .borders(Borders::ALL)
    .border_style(search_style)
    .title(" Search (/ to focus) ");

  let inner = search_block.inner(chunks[0]);

  // 搜索内容
  let search_text = if app.query.is_empty() && app.focus != Focus::Search {
    Span::styled("Type to search commands...", Style::default().fg(Color::DarkGray))
  } else {
    Span::raw(&app.query)
  };

  let search_input = Paragraph::new(search_text).block(search_block);
  frame.render_widget(search_input, chunks[0]);

  // 光标
  if app.focus == Focus::Search {
    frame.set_cursor_position((inner.x + app.cursor as u16, inner.y));
  }

  // 快捷键提示
  let hints = Paragraph::new(" [Tab] Switch  [F1] Help  [q] Quit")
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Right);
  
  // 垂直居中显示提示
  let hint_area = Rect {
    x: chunks[1].x,
    y: chunks[1].y + 1, // 垂直居中
    width: chunks[1].width,
    height: 1,
  };
  frame.render_widget(hints, hint_area);
}

/// 渲染主内容区
fn render_main(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
    .split(area);

  render_list(frame, app, chunks[0]);
  render_detail(frame, app, chunks[1]);
}

/// 渲染搜索结果列表
fn render_list(frame: &mut Frame, app: &App, area: Rect) {
  let list_style = if app.focus == Focus::List {
    Style::default().fg(Color::Yellow)
  } else {
    Style::default().fg(Color::Gray)
  };

  let title = if app.results.is_empty() {
    " Results ".to_string()
  } else {
    format!(" Results ({}) ", app.results.len())
  };

  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(list_style)
    .title(title);

  if app.results.is_empty() {
    let empty_text = if app.query.is_empty() {
      "Type to search"
    } else if app.loading {
      "Searching..."
    } else {
      "No results found"
    };
    let empty = Paragraph::new(empty_text)
      .style(Style::default().fg(Color::DarkGray))
      .block(block);
    frame.render_widget(empty, area);
    return;
  }

  let items: Vec<ListItem> = app
    .results
    .iter()
    .enumerate()
    .map(|(i, result)| {
      let style = if i == app.selected {
        Style::default()
          .bg(Color::Blue)
          .fg(Color::White)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::default()
      };

      let content = Line::from(vec![
        Span::styled(&result.name, style),
        Span::styled(
          format!(" - {}", truncate(&result.description, 30)),
          if i == app.selected {
            style
          } else {
            Style::default().fg(Color::DarkGray)
          },
        ),
      ]);

      ListItem::new(content)
    })
    .collect();

  let list = List::new(items).block(block).highlight_style(
    Style::default()
      .bg(Color::Blue)
      .add_modifier(Modifier::BOLD),
  );

  frame.render_widget(list, area);
}

/// 渲染命令详情
fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
  let detail_style = if app.focus == Focus::Detail {
    Style::default().fg(Color::Yellow)
  } else {
    Style::default().fg(Color::Gray)
  };

  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(detail_style)
    .title(" Details ");

  let Some((name, lang)) = app.selected_command() else {
    let empty = Paragraph::new("Select a command to view details")
      .style(Style::default().fg(Color::DarkGray))
      .block(block);
    frame.render_widget(empty, area);
    return;
  };

  let content = app
    .get_command_detail(name, lang)
    .unwrap_or_else(|| format!("Command not found: {} ({})", name, lang));

  // 简单的 Markdown 渲染
  let lines: Vec<Line> = content
    .lines()
    .map(|line| {
      if line.starts_with("# ") {
        Line::from(Span::styled(
          &line[2..],
          Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        ))
      } else if line.starts_with("## ") {
        Line::from(Span::styled(
          &line[3..],
          Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        ))
      } else if line.starts_with("```") {
        Line::from(Span::styled(
          "────────────────",
          Style::default().fg(Color::DarkGray),
        ))
      } else if line.starts_with("  ") || line.starts_with('\t') {
        // 代码行
        Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
      } else {
        Line::from(line)
      }
    })
    .collect();

  let paragraph = Paragraph::new(lines)
    .block(block)
    .wrap(Wrap { trim: false })
    .scroll((app.detail_scroll, 0));

  frame.render_widget(paragraph, area);
}

/// 渲染日志面板
fn render_log_panel(frame: &mut Frame, app: &App, area: Rect) {
  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Magenta))
    .title(" Debug Logs [F12 close] ");

  let logs = app.get_logs();
  let inner_height = area.height.saturating_sub(2) as usize;
  
  // 计算显示的日志范围（显示最新的日志）
  let start = if logs.len() > inner_height {
    logs.len() - inner_height
  } else {
    0
  };
  
  let lines: Vec<Line> = logs
    .iter()
    .skip(start)
    .map(|log| {
      let style = if log.contains("[ERROR]") {
        Style::default().fg(Color::Red)
      } else if log.contains("[WARN]") {
        Style::default().fg(Color::Yellow)
      } else if log.contains("[DEBUG]") {
        Style::default().fg(Color::DarkGray)
      } else {
        Style::default().fg(Color::Gray)
      };
      Line::from(Span::styled(log.clone(), style))
    })
    .collect();

  let paragraph = Paragraph::new(lines)
    .block(block)
    .wrap(Wrap { trim: true });

  frame.render_widget(paragraph, area);
}

/// 渲染状态栏
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
    .split(area);

  // 左侧状态
  let status = Paragraph::new(format!(" {} ", app.status))
    .style(Style::default().fg(Color::Cyan));
  frame.render_widget(status, chunks[0]);

  // 右侧：导航提示 + 版本信息
  let hints = Paragraph::new("[↑↓/jk] Nav  [Enter] View  Rust-powered CLI Cheatsheet ")
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Right);
  frame.render_widget(hints, chunks[1]);
}

/// 渲染帮助弹窗
fn render_help_popup(frame: &mut Frame) {
  let area = centered_rect(50, 60, frame.area());

  frame.render_widget(Clear, area);

  let help_text = vec![
    Line::from(Span::styled(
      "Keyboard Shortcuts",
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    )),
    Line::from(""),
    Line::from(vec![
      Span::styled("  /        ", Style::default().fg(Color::Yellow)),
      Span::raw("Focus search"),
    ]),
    Line::from(vec![
      Span::styled("  ↑↓ / jk  ", Style::default().fg(Color::Yellow)),
      Span::raw("Navigate"),
    ]),
    Line::from(vec![
      Span::styled("  Enter    ", Style::default().fg(Color::Yellow)),
      Span::raw("View details"),
    ]),
    Line::from(vec![
      Span::styled("  Tab      ", Style::default().fg(Color::Yellow)),
      Span::raw("Switch focus"),
    ]),
    Line::from(vec![
      Span::styled("  Esc      ", Style::default().fg(Color::Yellow)),
      Span::raw("Clear / Back"),
    ]),
    Line::from(vec![
      Span::styled("  PgUp/Dn  ", Style::default().fg(Color::Yellow)),
      Span::raw("Page up/down"),
    ]),
    Line::from(vec![
      Span::styled("  F1       ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle help"),
    ]),
    Line::from(vec![
      Span::styled("  F12      ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle debug logs"),
    ]),
    Line::from(vec![
      Span::styled("  q        ", Style::default().fg(Color::Yellow)),
      Span::raw("Quit (not in search)"),
    ]),
    Line::from(vec![
      Span::styled("  Ctrl+Q/C ", Style::default().fg(Color::Yellow)),
      Span::raw("Force quit"),
    ]),
    Line::from(""),
    Line::from(Span::styled(
      "Press any key to close",
      Style::default().fg(Color::DarkGray),
    )),
  ];

  let help = Paragraph::new(help_text)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help "),
    )
    .alignment(Alignment::Left);

  frame.render_widget(help, area);
}

/// 居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
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
