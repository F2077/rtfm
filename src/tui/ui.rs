use ratatui::{
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
  Frame,
};
use unicode_width::UnicodeWidthStr;

use super::app::{App, Focus, UiStyle};

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
pub fn render(frame: &mut Frame, app: &mut App) {
  match app.ui_style {
    UiStyle::Modern => render_modern(frame, app),
    UiStyle::Classic => render_classic(frame, app),
  }
}

/// Classic 风格渲染
fn render_classic(frame: &mut Frame, app: &mut App) {
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
  } else if app.show_logs {
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
    Span::styled(
      "Type to search commands...",
      Style::default().fg(Color::DarkGray),
    )
  } else {
    Span::raw(&app.query)
  };

  let search_input = Paragraph::new(search_text).block(search_block);
  frame.render_widget(search_input, chunks[0]);

  // 光标
  if app.focus == Focus::Search {
    let display_width = app.query[..app.cursor].width() as u16;
    frame.set_cursor_position((inner.x + display_width, inner.y));
  }

  // 快捷键提示
  let hints = Paragraph::new(" [Tab] Switch  [Ctrl+H] Help  [Esc] Back/Quit")
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
fn render_main(frame: &mut Frame, app: &mut App, area: Rect) {
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

      // Show full command name, let ratatui handle overflow
      let content = Line::from(Span::styled(&result.name, style));
      ListItem::new(content)
    })
    .collect();

  let list = List::new(items).block(block).highlight_style(
    Style::default()
      .bg(Color::Blue)
      .add_modifier(Modifier::BOLD),
  );

  // Use ListState for proper scrolling when selected item is out of view
  let mut list_state = ListState::default();
  list_state.select(Some(app.selected));
  frame.render_stateful_widget(list, area, &mut list_state);
}

/// 渲染命令详情
fn render_detail(frame: &mut Frame, app: &mut App, area: Rect) {
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
      if let Some(header) = line.strip_prefix("# ") {
        Line::from(Span::styled(
          header,
          Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        ))
      } else if let Some(header) = line.strip_prefix("## ") {
        Line::from(Span::styled(
          header,
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

  // Calculate and set max scroll
  let content_lines = lines.len() as u16;
  let visible_lines = area.height.saturating_sub(2); // subtract border
  app.set_detail_max_scroll(content_lines, visible_lines);

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
    .title(" Debug Logs [Ctrl+L close] ");

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

  let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

  frame.render_widget(paragraph, area);
}

/// 渲染状态栏
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  // When an item is selected, show full name + description using entire width
  if let Some((name, lang)) = app.selected_command() {
    let desc = app
      .results
      .get(app.selected)
      .map(|r| r.description.as_str())
      .unwrap_or("");

    // Calculate available space: total width - name - " [xx] - " (about 8 chars)
    let prefix = format!(" {} [{}]", name, lang);
    let prefix_len = prefix.chars().count();
    let available = (area.width as usize).saturating_sub(prefix_len + 4);

    let text = if available > 10 && !desc.is_empty() {
      format!("{} - {}", prefix, truncate(desc, available))
    } else {
      prefix
    };

    let status = Paragraph::new(text).style(Style::default().fg(Color::Cyan));
    frame.render_widget(status, area);
  } else {
    // No selection: show status on left, hints on right
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
      .split(area);

    let status = Paragraph::new(format!(" {}", app.status)).style(Style::default().fg(Color::Cyan));
    frame.render_widget(status, chunks[0]);

    let hints = Paragraph::new("[↑↓/jk] Nav  [Enter] View ")
      .style(Style::default().fg(Color::DarkGray))
      .alignment(Alignment::Right);
    frame.render_widget(hints, chunks[1]);
  }
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
      Span::raw("Navigate / Scroll"),
    ]),
    Line::from(vec![
      Span::styled("  ←→ / hl  ", Style::default().fg(Color::Yellow)),
      Span::raw("Switch results (Modern) / Focus (Classic)"),
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
      Span::raw("Clear / Back / Quit"),
    ]),
    Line::from(vec![
      Span::styled("  PgUp/Dn  ", Style::default().fg(Color::Yellow)),
      Span::raw("Page up/down"),
    ]),
    Line::from(vec![
      Span::styled("  g / G    ", Style::default().fg(Color::Yellow)),
      Span::raw("Jump to first/last"),
    ]),
    Line::from(vec![
      Span::styled("  Ctrl+H   ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle help (or ? outside search)"),
    ]),
    Line::from(vec![
      Span::styled("  Ctrl+T   ", Style::default().fg(Color::Yellow)),
      Span::raw("Switch UI style (Modern/Classic)"),
    ]),
    Line::from(vec![
      Span::styled("  Ctrl+L   ", Style::default().fg(Color::Yellow)),
      Span::raw("Toggle debug logs (requires --debug)"),
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

// ============================================================================
// Modern 风格渲染 - 简约通栏布局
// ============================================================================

/// Modern 风格渲染
/// 布局：Logo（通栏）+ 搜索框（通栏）+ 结果详情（通栏，单条）+ [日志]
fn render_modern(frame: &mut Frame, app: &mut App) {
  let area = frame.area();

  // 检查是否有足够空间显示 Logo
  let min_height_for_logo = if app.show_logs { 20 } else { 15 };
  let show_logo = area.height >= min_height_for_logo;

  // 布局约束
  let constraints = if show_logo {
    if app.show_logs {
      vec![
        Constraint::Length(LOGO_HEIGHT), // Logo
        Constraint::Length(3),           // 搜索框
        Constraint::Min(5),              // 结果详情
        Constraint::Length(8),           // 日志
      ]
    } else {
      vec![
        Constraint::Length(LOGO_HEIGHT), // Logo
        Constraint::Length(3),           // 搜索框
        Constraint::Min(5),              // 结果详情
      ]
    }
  } else if app.show_logs {
    vec![
      Constraint::Length(3), // 搜索框
      Constraint::Min(5),    // 结果详情
      Constraint::Length(8), // 日志
    ]
  } else {
    vec![
      Constraint::Length(3), // 搜索框
      Constraint::Min(5),    // 结果详情
    ]
  };

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints)
    .split(area);

  let mut idx = 0;

  // Logo
  if show_logo {
    render_modern_logo(frame, chunks[idx]);
    idx += 1;
  }

  // 搜索框
  render_modern_search(frame, app, chunks[idx]);
  idx += 1;

  // 结果详情（单条显示）
  render_modern_result(frame, app, chunks[idx]);
  idx += 1;

  // 日志面板
  if app.show_logs {
    render_modern_logs(frame, app, chunks[idx]);
  }

  // 帮助弹窗
  if app.show_help {
    render_help_popup(frame);
  }
}

/// Modern Logo 渲染（居中显示）
fn render_modern_logo(frame: &mut Frame, area: Rect) {
  let lines: Vec<Line> = LOGO
    .iter()
    .map(|line| {
      Line::from(Span::styled(
        *line,
        Style::default()
          .fg(Color::Rgb(255, 100, 100))
          .add_modifier(Modifier::BOLD),
      ))
    })
    .collect();

  let logo = Paragraph::new(lines).alignment(Alignment::Left);
  frame.render_widget(logo, area);
}

/// Modern 搜索框（简约风格，通栏）
fn render_modern_search(frame: &mut Frame, app: &App, area: Rect) {
  // 搜索框样式
  let border_color = if app.focus == Focus::Search {
    Color::Rgb(100, 200, 255)
  } else {
    Color::DarkGray
  };

  let block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(border_color))
    .title(Span::styled(
      " Search ",
      Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD),
    ));

  let inner = block.inner(area);

  // 搜索内容
  let prefix = Span::styled("> ", Style::default().fg(Color::Rgb(100, 200, 255)));
  let content = if app.query.is_empty() && app.focus != Focus::Search {
    Span::styled(
      "Type to search... (↑↓ to navigate results)",
      Style::default().fg(Color::DarkGray),
    )
  } else {
    Span::styled(&app.query, Style::default().fg(Color::White))
  };

  // 右侧提示
  let hint = " [Ctrl+H] Help  [Esc] Quit ";
  let hint_width = hint.len() as u16;

  let search = Paragraph::new(Line::from(vec![prefix, content])).block(block);
  frame.render_widget(search, area);

  // 右侧提示（在边框内）
  if area.width > hint_width + 20 {
    let hint_area = Rect {
      x: area.x + area.width - hint_width - 1,
      y: area.y,
      width: hint_width,
      height: 1,
    };
    let hint_widget = Paragraph::new(hint).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint_widget, hint_area);
  }

  // 光标
  if app.focus == Focus::Search {
    let display_width = app.query[..app.cursor].width() as u16;
    frame.set_cursor_position((inner.x + 2 + display_width, inner.y));
  }
}

/// Modern 结果显示（单条详情，通栏）
fn render_modern_result(frame: &mut Frame, app: &mut App, area: Rect) {
  let border_color = if app.focus == Focus::List || app.focus == Focus::Detail {
    Color::Rgb(100, 200, 255)
  } else {
    Color::DarkGray
  };

  // 标题显示当前位置
  let title = if app.results.is_empty() {
    " Result ".to_string()
  } else {
    format!(" Result [{}/{}] ", app.selected + 1, app.results.len())
  };

  let block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(border_color))
    .title(Span::styled(
      title,
      Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD),
    ));

  // 无结果时的提示
  if app.results.is_empty() {
    let empty_text = if app.query.is_empty() {
      vec![
        Line::from(""),
        Line::from(Span::styled(
          "  Start typing to search commands...",
          Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
          "  Tips: Use ↑↓ or j/k to navigate results",
          Style::default().fg(Color::DarkGray),
        )),
      ]
    } else if app.loading {
      vec![
        Line::from(""),
        Line::from(Span::styled(
          "  Searching...",
          Style::default().fg(Color::Yellow),
        )),
      ]
    } else {
      vec![
        Line::from(""),
        Line::from(Span::styled(
          "  No results found",
          Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
          "  Try a different search term or run 'rtfm update'",
          Style::default().fg(Color::DarkGray),
        )),
      ]
    };
    let empty = Paragraph::new(empty_text).block(block);
    frame.render_widget(empty, area);
    return;
  }

  // 获取当前选中的命令
  let result = &app.results[app.selected];
  let content = app
    .get_command_detail(&result.name, &result.lang)
    .unwrap_or_else(|| format!("Command not found: {}", result.name));

  // 渲染命令详情（Markdown 风格）
  let mut lines: Vec<Line> = Vec::new();

  for line in content.lines() {
    if let Some(h) = line.strip_prefix("# ") {
      // 一级标题：命令名
      lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
          h,
          Style::default()
            .fg(Color::Rgb(100, 200, 255))
            .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
          format!("  [{}]", result.lang),
          Style::default().fg(Color::DarkGray),
        ),
      ]));
    } else if let Some(h) = line.strip_prefix("## ") {
      // 二级标题：示例描述
      lines.push(Line::from(""));
      lines.push(Line::from(vec![
        Span::styled("  → ", Style::default().fg(Color::Green)),
        Span::styled(
          h,
          Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        ),
      ]));
    } else if line.starts_with("```") {
      // 代码块分隔符（跳过）
    } else if line.starts_with("  ") || line.starts_with('\t') {
      // 代码行
      lines.push(Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(line.trim(), Style::default().fg(Color::Yellow)),
      ]));
    } else if !line.trim().is_empty() {
      // 普通文本（描述）
      lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(line, Style::default().fg(Color::White)),
      ]));
    }
  }

  // 底部导航提示
  lines.push(Line::from(""));
  lines.push(Line::from(Span::styled(
    "  ↑↓ Scroll  ←→ Switch result  / Search  ? Help",
    Style::default().fg(Color::DarkGray),
  )));

  // 计算滚动
  let content_lines = lines.len() as u16;
  let visible_lines = area.height.saturating_sub(2);
  app.set_detail_max_scroll(content_lines, visible_lines);

  let paragraph = Paragraph::new(lines)
    .block(block)
    .wrap(Wrap { trim: false })
    .scroll((app.detail_scroll, 0));

  frame.render_widget(paragraph, area);
}

/// Modern 日志面板
fn render_modern_logs(frame: &mut Frame, app: &App, area: Rect) {
  let block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(Color::Magenta))
    .title(Span::styled(
      " Logs [Ctrl+L close] ",
      Style::default().fg(Color::Magenta),
    ));

  let logs = app.get_logs();
  let inner_height = area.height.saturating_sub(2) as usize;
  let start = logs.len().saturating_sub(inner_height);

  let lines: Vec<Line> = logs
    .iter()
    .skip(start)
    .map(|log| {
      let style = if log.contains("[ERROR]") {
        Style::default().fg(Color::Red)
      } else if log.contains("[WARN]") {
        Style::default().fg(Color::Yellow)
      } else {
        Style::default().fg(Color::DarkGray)
      };
      Line::from(Span::styled(format!("  {}", log), style))
    })
    .collect();

  let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
  frame.render_widget(paragraph, area);
}
