//! Learn module - capture and parse command help
//! 
//! 跨平台帮助获取策略：
//! - Windows: --help, -h, /?, Get-Help (PowerShell), help (cmd)
//! - macOS: --help, -h, man
//! - Linux: --help, -h, man

use std::process::Command;

use crate::storage::{Command as StorageCommand, Example};

/// 获取命令帮助的统一入口（跨平台自适应）
/// 返回 (内容, 来源) 或错误
pub fn get_help_output(cmd: &str) -> anyhow::Result<(String, String)> {
    // 根据平台选择帮助获取策略
    #[cfg(target_os = "windows")]
    {
        get_help_windows(cmd)
    }

    #[cfg(target_os = "macos")]
    {
        get_help_unix(cmd)
    }

    #[cfg(target_os = "linux")]
    {
        get_help_unix(cmd)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        get_help_unix(cmd)
    }
}

/// Windows 平台帮助获取
/// 尝试顺序: --help -> -h -> /? -> Get-Help (PowerShell)
#[cfg(target_os = "windows")]
fn get_help_windows(cmd: &str) -> anyhow::Result<(String, String)> {
    // 1. 尝试 --help（跨平台通用）
    if let Ok(result) = try_help_flag(cmd, "--help") {
        return Ok(result);
    }

    // 2. 尝试 -h
    if let Ok(result) = try_help_flag(cmd, "-h") {
        return Ok(result);
    }

    // 3. 尝试 /? (Windows 传统风格)
    if let Ok(result) = try_help_flag(cmd, "/?") {
        return Ok(result);
    }

    // 4. 尝试 PowerShell Get-Help（对 PowerShell cmdlet 有效）
    if let Ok(result) = get_powershell_help(cmd) {
        return Ok(result);
    }

    // 5. 尝试 cmd 的 help 命令（对内置命令有效）
    if let Ok(result) = get_cmd_help(cmd) {
        return Ok(result);
    }

    // 检查命令是否存在
    let check = Command::new("where")
        .arg(cmd)
        .output();

    match check {
        Ok(output) if output.status.success() => {
            anyhow::bail!("Command '{}' exists but no help output available", cmd)
        }
        _ => {
            anyhow::bail!("Command '{}' not found (program not found)", cmd)
        }
    }
}

/// Unix 平台帮助获取 (Linux/macOS)
/// 尝试顺序: --help -> -h
#[cfg(any(target_os = "linux", target_os = "macos", not(target_os = "windows")))]
fn get_help_unix(cmd: &str) -> anyhow::Result<(String, String)> {
    // 1. 尝试 --help
    if let Ok(result) = try_help_flag(cmd, "--help") {
        return Ok(result);
    }

    // 2. 尝试 -h
    if let Ok(result) = try_help_flag(cmd, "-h") {
        return Ok(result);
    }

    // 检查命令是否存在
    let check = Command::new("which")
        .arg(cmd)
        .output();

    match check {
        Ok(output) if output.status.success() => {
            anyhow::bail!("Command '{}' exists but --help/-h didn't provide usable output", cmd)
        }
        _ => {
            anyhow::bail!("Command '{}' not found (program not found)", cmd)
        }
    }
}

/// 尝试使用指定的帮助标志获取帮助
fn try_help_flag(cmd: &str, flag: &str) -> anyhow::Result<(String, String)> {
    let output = Command::new(cmd)
        .arg(flag)
        .output();

    match output {
        Ok(output) => {
            // 检查 stdout
            if output.status.success() || !output.stdout.is_empty() {
                let content = String::from_utf8_lossy(&output.stdout).to_string();
                if is_valid_help_content(&content) {
                    return Ok((content, flag.to_string()));
                }
            }
            // 有些命令把帮助输出到 stderr
            if !output.stderr.is_empty() {
                let content = String::from_utf8_lossy(&output.stderr).to_string();
                if is_valid_help_content(&content) {
                    return Ok((content, format!("{} (stderr)", flag)));
                }
            }
            anyhow::bail!("No usable output from {} {}", cmd, flag)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::bail!("Command '{}' not found (program not found)", cmd);
            }
            anyhow::bail!("Failed to execute '{} {}': {}", cmd, flag, e)
        }
    }
}

/// 检查内容是否是有效的帮助文本
fn is_valid_help_content(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }
    // 至少包含一些帮助相关的关键词
    let lower = trimmed.to_lowercase();
    lower.contains("usage") 
        || lower.contains("options")
        || lower.contains("help")
        || lower.contains("commands")
        || lower.contains("synopsis")
        || lower.contains("description")
        || trimmed.len() > 50 // 或者内容足够长
}

/// Windows: 使用 PowerShell Get-Help 获取帮助
#[cfg(target_os = "windows")]
fn get_powershell_help(cmd: &str) -> anyhow::Result<(String, String)> {
    // Get-Help 可以获取 PowerShell cmdlet 和一些外部命令的帮助
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("Get-Help {} -ErrorAction SilentlyContinue | Out-String -Width 120", cmd)])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed = content.trim();
            
            // 完全没有输出
            if trimmed.is_empty() {
                anyhow::bail!("Get-Help found no help for '{}'", cmd)
            }
            
            // 完全找不到帮助主题（只有错误消息，没有实际内容）
            if (trimmed.contains("No help topic found") || trimmed.starts_with("Get-Help cannot"))
                && trimmed.len() < 200 {
                anyhow::bail!("Get-Help found no help for '{}'", cmd)
            }
            
            // 有 NAME/SYNTAX 等有效内容，即使有警告也接受
            Ok((content, "Get-Help (PowerShell)".to_string()))
        }
        Ok(_) => anyhow::bail!("Get-Help failed for '{}'", cmd),
        Err(e) => anyhow::bail!("Failed to run PowerShell Get-Help: {}", e),
    }
}

/// Windows: 使用 cmd 的 help 命令获取内置命令帮助
#[cfg(target_os = "windows")]
fn get_cmd_help(cmd: &str) -> anyhow::Result<(String, String)> {
    // help 命令只对 cmd 内置命令有效（如 dir, copy, del 等）
    let output = Command::new("cmd")
        .args(["/c", "help", cmd])
        .output();

    match output {
        Ok(output) => {
            // cmd help 有时返回非零退出码但仍有有效输出，所以检查内容而非退出码
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            if is_valid_help_content(&content) && !content.contains("is not supported") {
                return Ok((content, "help (cmd)".to_string()));
            }
            anyhow::bail!("No cmd help for '{}'", cmd)
        }
        Err(e) => anyhow::bail!("Failed to run cmd help: {}", e),
    }
}

/// 获取命令的 man 页面（跨平台自适应）
/// - Linux: 标准 man 命令
/// - macOS: man 命令（参数格式略有不同）
/// - Windows: 不支持 man，返回提示
pub fn get_man_page(cmd: &str) -> anyhow::Result<(String, String)> {
    #[cfg(target_os = "windows")]
    {
        let _ = cmd; // 避免未使用警告
        // Windows 没有 man 命令，提示使用其他方式
        anyhow::bail!("'man' is not available on Windows. Use --help or Get-Help instead.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        get_man_page_unix(cmd)
    }
}

/// Unix 平台的 man 页面获取
#[cfg(not(target_os = "windows"))]
fn get_man_page_unix(cmd: &str) -> anyhow::Result<(String, String)> {
    // macOS 和 Linux 都使用 man 命令，但环境变量设置方式相同
    let output = Command::new("man")
        .env("MANPAGER", "cat")
        .env("MANWIDTH", "80")
        // macOS 上某些情况需要禁用颜色
        .env("GROFF_NO_SGR", "1")
        .arg(cmd)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            // 移除 ANSI 转义序列和 backspace 效果
            let clean = strip_ansi_codes(&content);
            if clean.trim().is_empty() {
                anyhow::bail!("man page for '{}' is empty", cmd);
            }
            Ok((clean, "man".to_string()))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_lower = stderr.to_lowercase();
            if stderr_lower.contains("no manual entry") || stderr_lower.contains("no entry") {
                anyhow::bail!("No man page for '{}'", cmd)
            }
            anyhow::bail!("man failed for '{}': {}", cmd, stderr.trim())
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::bail!("'man' command not available (program not found)");
            }
            anyhow::bail!("Failed to run man: {}", e)
        }
    }
}

/// 移除 ANSI 转义序列（仅 Unix 平台使用）
#[cfg(not(target_os = "windows"))]
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // 跳过 ESC 序列
            if chars.peek() == Some(&'[') {
                chars.next();
                // 跳过直到遇到字母
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else if c == '\x08' {
            // Backspace (用于粗体/下划线效果)
            result.pop();
        } else {
            result.push(c);
        }
    }

    result
}

/// 解析帮助内容为结构化命令
pub fn parse_help_content(name: &str, content: &str, source: &str) -> StorageCommand {
    let lines: Vec<&str> = content.lines().collect();

    // 提取描述（通常在开头几行）
    let description = extract_description(&lines, name);

    // 提取示例
    let examples = extract_examples(&lines, name);

    StorageCommand {
        name: name.to_string(),
        description,
        category: "local".to_string(),
        platform: get_platform(),
        lang: "local".to_string(),
        examples,
        content: format!("Source: {}\n\n{}", source, content),
    }
}

/// 提取描述
fn extract_description(lines: &[&str], name: &str) -> String {
    let mut description = String::new();

    for line in lines.iter().take(20) {
        let line = line.trim();

        // 跳过空行
        if line.is_empty() {
            if !description.is_empty() {
                break; // 描述结束
            }
            continue;
        }

        // 跳过 Usage 行
        if line.to_lowercase().starts_with("usage") {
            continue;
        }

        // 跳过命令名开头的行（通常是 NAME 部分后的格式化行）
        if line.starts_with(name) && line.contains(" - ") {
            // man 格式: "command - description"
            if let Some(desc) = line.split(" - ").nth(1) {
                return desc.trim().to_string();
            }
        }

        // 收集描述行
        if !line.starts_with('-') && !line.starts_with('[') && !line.contains("--") {
            if description.is_empty() {
                description = line.to_string();
            } else {
                description.push(' ');
                description.push_str(line);
            }

            // 描述通常不超过 200 字符
            if description.len() > 200 {
                break;
            }
        }
    }

    if description.is_empty() {
        format!("{} command (learned from local system)", name)
    } else {
        // 清理描述
        description
            .replace("  ", " ")
            .trim()
            .to_string()
    }
}

/// 提取示例
fn extract_examples(lines: &[&str], name: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    let mut in_examples = false;
    let mut current_desc = String::new();

    for line in lines {
        let line_lower = line.to_lowercase();
        let trimmed = line.trim();

        // 检测示例部分
        if line_lower.contains("example") || line_lower.contains("synopsis") {
            in_examples = true;
            continue;
        }

        // 检测示例结束
        if in_examples && (line_lower.starts_with("options") 
            || line_lower.starts_with("description")
            || line_lower.starts_with("see also")) {
            in_examples = false;
        }

        // 在示例部分或整个内容中查找命令行
        if trimmed.starts_with(name) || trimmed.starts_with(&format!("$ {}", name)) {
            let code = trimmed.trim_start_matches("$ ").to_string();
            let desc = if current_desc.is_empty() {
                extract_inline_description(trimmed)
            } else {
                std::mem::take(&mut current_desc)
            };

            examples.push(Example {
                description: desc,
                code,
            });

            if examples.len() >= 10 {
                break;
            }
        } else if !trimmed.is_empty() 
            && !trimmed.starts_with('-') 
            && !trimmed.starts_with('[')
            && trimmed.len() < 100 
        {
            // 可能是示例描述
            current_desc = trimmed.to_string();
        }
    }

    // 如果没找到示例，提取常用选项作为示例
    if examples.is_empty() {
        examples = extract_options_as_examples(lines, name);
    }

    examples
}

/// 从选项中提取示例
fn extract_options_as_examples(lines: &[&str], name: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    let mut found_options = false;

    for line in lines {
        let trimmed = line.trim();

        // 检测选项部分
        if trimmed.to_lowercase().starts_with("options") 
            || trimmed.to_lowercase().starts_with("flags") {
            found_options = true;
            continue;
        }

        if found_options && trimmed.starts_with('-') {
            // 解析选项行，如 "-v, --verbose  Enable verbose mode"
            if let Some((opt, desc)) = parse_option_line(trimmed) {
                examples.push(Example {
                    description: desc,
                    code: format!("{} {}", name, opt),
                });

                if examples.len() >= 5 {
                    break;
                }
            }
        }
    }

    examples
}

/// 解析选项行
fn parse_option_line(line: &str) -> Option<(String, String)> {
    // 格式: "-v, --verbose  Description"
    let parts: Vec<&str> = line.splitn(2, "  ").collect();

    if parts.len() == 2 {
        let opt = parts[0].trim();
        let desc = parts[1].trim();

        // 提取主要选项（优先长选项）
        let main_opt = opt
            .split(',')
            .map(|s| s.trim())
            .find(|s| s.starts_with("--"))
            .or_else(|| opt.split(',').next().map(|s| s.trim()))
            .unwrap_or(opt);

        if !main_opt.is_empty() && !desc.is_empty() {
            return Some((main_opt.to_string(), desc.to_string()));
        }
    }

    None
}

/// 提取行内描述
fn extract_inline_description(line: &str) -> String {
    // 尝试提取 # comment 或命令后的描述
    if let Some(idx) = line.find('#') {
        return line[idx + 1..].trim().to_string();
    }

    "Example usage".to_string()
}

/// 获取当前平台
fn get_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "osx".to_string()
    } else {
        "linux".to_string()
    }
}

/// 获取指定 section 的所有 man 页面列表
/// 返回 (命令名, 描述) 列表
/// 注意：仅在 Unix 系统 (Linux/macOS) 上可用
pub fn list_man_pages(section: &str) -> anyhow::Result<Vec<(String, String)>> {
    #[cfg(target_os = "windows")]
    {
        let _ = section; // 避免未使用警告
        anyhow::bail!("'man' is not available on Windows. This feature only works on Linux/macOS.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        list_man_pages_unix(section)
    }
}

/// Unix 平台的 man 页面列表
#[cfg(not(target_os = "windows"))]
fn list_man_pages_unix(section: &str) -> anyhow::Result<Vec<(String, String)>> {
    // Linux: man -k -s <section> .
    // macOS: man -k . (然后过滤 section)
    
    // 先尝试 Linux 风格
    let output = Command::new("man")
        .arg("-k")
        .arg("-s")
        .arg(section)
        .arg(".")
        .output();

    let output = match output {
        Ok(o) if o.status.success() && !o.stdout.is_empty() => o,
        _ => {
            // 尝试 macOS 风格（不支持 -s 参数）
            let mac_output = Command::new("man")
                .arg("-k")
                .arg(".")
                .output();
            
            match mac_output {
                Ok(o) if o.status.success() => o,
                Ok(o) => {
                    // 尝试 apropos
                    let apropos = Command::new("apropos")
                        .arg(".")
                        .output()?;
                    if apropos.status.success() {
                        apropos
                    } else {
                        anyhow::bail!("Failed to list man pages: {}", String::from_utf8_lossy(&o.stderr));
                    }
                }
                Err(e) => anyhow::bail!("man/apropos not available: {}", e),
            }
        }
    };

    let content = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in content.lines() {
        // 解析格式: "command (section) - description"
        // 或: "command, alias (section) - description"
        if let Some((name, desc)) = parse_man_list_line(line, section) {
            results.push((name, desc));
        }
    }

    Ok(results)
}

/// 解析 man -k 输出行（仅 Unix 平台使用）
#[cfg(not(target_os = "windows"))]
fn parse_man_list_line(line: &str, section: &str) -> Option<(String, String)> {
    // 格式: "command (1) - description"
    // 或: "command, alias (1) - description"
    
    let section_marker = format!("({})", section);
    
    // 检查是否包含目标 section
    if !line.contains(&section_marker) && !line.contains(&format!("({},", section)) {
        return None;
    }

    // 提取命令名（第一个词，或逗号前的部分）
    let name = line.split(|c| c == '(' || c == ',' || c == ' ')
        .next()?
        .trim();

    if name.is_empty() {
        return None;
    }

    // 提取描述（" - " 后面的部分）
    let desc = line.split(" - ")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();

    Some((name.to_string(), desc))
}

/// 获取指定 section 的单个 man 页面
/// 注意：仅在 Unix 系统 (Linux/macOS) 上可用
pub fn get_man_page_with_section(cmd: &str, section: &str) -> anyhow::Result<(String, String)> {
    #[cfg(target_os = "windows")]
    {
        let _ = (cmd, section); // 避免未使用警告
        anyhow::bail!("'man' is not available on Windows");
    }

    #[cfg(not(target_os = "windows"))]
    {
        get_man_page_with_section_unix(cmd, section)
    }
}

/// Unix 平台获取指定 section 的 man 页面
#[cfg(not(target_os = "windows"))]
fn get_man_page_with_section_unix(cmd: &str, section: &str) -> anyhow::Result<(String, String)> {
    let output = Command::new("man")
        .env("MANPAGER", "cat")
        .env("MANWIDTH", "80")
        .env("GROFF_NO_SGR", "1") // macOS 禁用颜色
        .arg(section)
        .arg(cmd)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            let clean = strip_ansi_codes(&content);
            if clean.trim().is_empty() {
                anyhow::bail!("man page for '{}({})' is empty", cmd, section);
            }
            Ok((clean, format!("man({})", section)))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("No man page for '{}' in section {}: {}", cmd, section, stderr.trim())
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::bail!("'man' command not available (program not found)");
            }
            anyhow::bail!("Failed to run man: {}", e)
        }
    }
}

/// 列出可学习的命令（跨平台）
/// 返回 (命令名, 描述) 列表
pub fn list_available_commands(source: &str) -> anyhow::Result<Vec<(String, String)>> {
    match source {
        "powershell" => list_powershell_cmdlets(),
        "path" => list_path_commands(),
        #[cfg(not(target_os = "windows"))]
        "man" => list_man_pages("1"),
        #[cfg(target_os = "windows")]
        "man" => anyhow::bail!("'man' is not available on Windows. Use 'powershell' or 'path' instead."),
        "auto" => {
            #[cfg(target_os = "windows")]
            {
                // Windows 默认使用 PowerShell
                list_powershell_cmdlets()
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Unix 默认使用 man
                list_man_pages("1")
            }
        }
        _ => anyhow::bail!("Unknown source '{}'. Use 'man', 'powershell', 'path', or 'auto'.", source),
    }
}

/// 列出 PowerShell cmdlet
fn list_powershell_cmdlets() -> anyhow::Result<Vec<(String, String)>> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("PowerShell cmdlet listing is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        list_powershell_cmdlets_windows()
    }
}

/// Windows: 列出 PowerShell cmdlet
#[cfg(target_os = "windows")]
fn list_powershell_cmdlets_windows() -> anyhow::Result<Vec<(String, String)>> {
    println!("Listing PowerShell cmdlets...");
    
    // 获取所有 cmdlet，只取名称和简介
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Command -CommandType Cmdlet,Function | Select-Object -Property Name | ForEach-Object { $_.Name }",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);
            let commands: Vec<(String, String)> = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|name| (name.trim().to_string(), "PowerShell cmdlet".to_string()))
                .collect();
            
            if commands.is_empty() {
                anyhow::bail!("No PowerShell cmdlets found");
            }
            
            Ok(commands)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to list PowerShell cmdlets: {}", stderr.trim())
        }
        Err(e) => anyhow::bail!("Failed to run PowerShell: {}", e),
    }
}

/// 列出 PATH 中的可执行文件
fn list_path_commands() -> anyhow::Result<Vec<(String, String)>> {
    #[cfg(target_os = "windows")]
    {
        list_path_commands_windows()
    }

    #[cfg(not(target_os = "windows"))]
    {
        list_path_commands_unix()
    }
}

/// Windows: 列出 PATH 中的可执行文件
#[cfg(target_os = "windows")]
fn list_path_commands_windows() -> anyhow::Result<Vec<(String, String)>> {
    use std::collections::HashSet;
    
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut commands = HashSet::new();
    
    for dir in path_var.split(';') {
        let dir_path = std::path::Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if ext_lower == "exe" || ext_lower == "cmd" || ext_lower == "bat" {
                        if let Some(name) = path.file_stem() {
                            commands.insert(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    let mut result: Vec<_> = commands
        .into_iter()
        .map(|name| (name, "PATH executable".to_string()))
        .collect();
    
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Unix: 列出 PATH 中的可执行文件
#[cfg(not(target_os = "windows"))]
fn list_path_commands_unix() -> anyhow::Result<Vec<(String, String)>> {
    use std::collections::HashSet;
    use std::os::unix::fs::PermissionsExt;
    
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut commands = HashSet::new();
    
    for dir in path_var.split(':') {
        let dir_path = std::path::Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    // 检查是否可执行
                    if let Ok(metadata) = path.metadata() {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            if let Some(name) = path.file_name() {
                                commands.insert(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut result: Vec<_> = commands
        .into_iter()
        .map(|name| (name, "PATH executable".to_string()))
        .collect();
    
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        let input = "\x1b[1mBold\x1b[0m text";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Bold text");
    }

    #[test]
    fn test_parse_option_line() {
        let line = "-v, --verbose  Enable verbose output";
        let (opt, desc) = parse_option_line(line).unwrap();
        assert_eq!(opt, "--verbose");
        assert_eq!(desc, "Enable verbose output");
    }

    #[test]
    fn test_parse_help_content() {
        let content = r#"
mycmd - A test command

Usage: mycmd [OPTIONS] <FILE>

Options:
  -v, --verbose  Enable verbose output
  -h, --help     Show help

Examples:
  mycmd file.txt
  mycmd -v file.txt
"#;
        let cmd = parse_help_content("mycmd", content, "--help");
        assert_eq!(cmd.name, "mycmd");
        assert!(!cmd.description.is_empty());
    }

    #[test]
    fn test_get_platform() {
        let platform = get_platform();
        assert!(["linux", "osx", "windows"].contains(&platform.as_str()));
    }
}
