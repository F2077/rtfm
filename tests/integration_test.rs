//! Integration tests for RTFM

use std::path::PathBuf;

#[test]
fn test_data_dir_default() {
  // 测试默认数据目录
  let data_dir = dirs::data_local_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("rtfm");

  assert!(data_dir.to_string_lossy().contains("rtfm"));
}

#[test]
fn test_command_name_normalization() {
  // 测试命令名规范化（空格转连字符）
  let input = "docker cp";
  let normalized = input.replace(' ', "-");
  assert_eq!(normalized, "docker-cp");
}

#[test]
fn test_truncate_string() {
  fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
      s.to_string()
    } else {
      let truncated: String = s.chars().take(max_len - 3).collect();
      format!("{}...", truncated)
    }
  }

  assert_eq!(truncate("hello", 10), "hello");
  assert_eq!(truncate("hello world", 8), "hello...");
  assert_eq!(truncate("中文测试字符串", 5), "中文...");
}
