# RTFM - Rust 新手开发指南

本指南面向没有 Rust 经验但希望参与 RTFM 项目开发的开发者。

## 目录

1. [环境准备](#环境准备)
2. [Rust 基础概念](#rust-基础概念)
3. [项目结构解析](#项目结构解析)
4. [核心代码阅读](#核心代码阅读)
5. [常见任务](#常见任务)
6. [调试技巧](#调试技巧)
7. [常见问题](#常见问题)

---

## 环境准备

### 安装 Rust

```bash
# 官方安装脚本（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows 用户下载安装器
# https://www.rust-lang.org/tools/install
```

安装后验证：

```bash
rustc --version   # 编译器版本
cargo --version   # 包管理器版本
```

### 配置 IDE

**推荐: VS Code + rust-analyzer**

1. 安装 VS Code
2. 安装扩展: `rust-analyzer`
3. 可选扩展: `Even Better TOML`, `crates`

**其他选择:**
- IntelliJ IDEA + Rust 插件
- Neovim + rust-analyzer LSP

### 克隆并运行项目

```bash
git clone <repository-url>
cd rtfm

# 编译并运行（首次会下载依赖，需要几分钟）
cargo run

# 运行测试
cargo test
```

---

## Rust 基础概念

### 所有权 (Ownership)

这是 Rust 最独特的概念，也是新手最大的障碍。

```rust
fn main() {
    let s1 = String::from("hello");  // s1 拥有这个字符串
    let s2 = s1;                      // 所有权转移给 s2
    // println!("{}", s1);            // 错误! s1 不再有效
    println!("{}", s2);               // OK
}
```

**项目中的例子:**

```rust
// src/storage/mod.rs
pub fn save_command(&self, cmd: &Command) -> Result<()> {
    // 注意参数是 &Command（借用），不是 Command（所有权转移）
    // 这样调用者仍然可以继续使用 cmd
    let value = serde_json::to_vec(cmd)?;
    // ...
}
```

### 借用 (Borrowing)

```rust
fn main() {
    let s = String::from("hello");
    
    // 不可变借用 - 可以同时有多个
    let len = calculate_length(&s);
    println!("Length of '{}' is {}", s, len);
    
    // 可变借用 - 同一时间只能有一个
    let mut s2 = String::from("hello");
    change(&mut s2);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

### Result 和错误处理

Rust 没有异常，使用 `Result` 类型处理错误：

```rust
// Result 有两个变体: Ok(成功值) 和 Err(错误)
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

fn main() {
    // 方式1: match
    match read_file("config.toml") {
        Ok(content) => println!("Content: {}", content),
        Err(e) => println!("Error: {}", e),
    }
    
    // 方式2: ? 操作符（在返回 Result 的函数中使用）
    // let content = read_file("config.toml")?;
    
    // 方式3: unwrap（仅用于示例/测试，生产代码避免使用）
    // let content = read_file("config.toml").unwrap();
}
```

**项目中的例子:**

```rust
// src/main.rs
async fn run_update(force: bool, config: &AppConfig) -> anyhow::Result<()> {
    // anyhow::Result 是一个便利类型，可以包装任何错误
    let update_info = update::check_github_release(&config.update).await?;
    //                                                              ^^ ? 表示：如果是错误就提前返回
    Ok(())
}
```

### Option 类型

表示"可能有值，可能没有"：

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

fn main() {
    match find_user(1) {
        Some(name) => println!("Found: {}", name),
        None => println!("Not found"),
    }
    
    // 或使用 if let
    if let Some(name) = find_user(1) {
        println!("Found: {}", name);
    }
}
```

### 结构体 (Struct)

```rust
// 定义
struct Command {
    name: String,
    description: String,
}

// 实现方法
impl Command {
    // 关联函数（类似静态方法）
    fn new(name: String, description: String) -> Self {
        Command { name, description }
    }
    
    // 实例方法，&self 是不可变借用
    fn display(&self) {
        println!("{}: {}", self.name, self.description);
    }
}
```

### 模块系统

```
src/
├── main.rs       # 主入口
├── cli/
│   └── mod.rs    # mod cli 的实现
└── storage/
    └── mod.rs    # mod storage 的实现
```

```rust
// main.rs
mod cli;      // 声明 cli 模块
mod storage;  // 声明 storage 模块

use cli::Cli;  // 使用 cli 模块中的 Cli
```

### async/await

异步编程用于 I/O 密集型操作：

```rust
// async 函数返回 Future，需要 await 来执行
async fn fetch_data() -> String {
    // 模拟网络请求
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    String::from("data")
}

#[tokio::main]  // 启动 tokio 运行时
async fn main() {
    let data = fetch_data().await;  // 等待 Future 完成
    println!("{}", data);
}
```

---

## 项目结构解析

### Cargo.toml

项目配置文件，类似 `package.json`：

```toml
[package]
name = "rtfm"           # 包名
version = "0.1.0"       # 版本
edition = "2021"        # Rust 版本

[dependencies]          # 运行时依赖
ratatui = "0.29"        # TUI 框架
clap = { version = "4.5", features = ["derive"] }  # CLI 框架

[dev-dependencies]      # 测试依赖
criterion = "0.5"       # 基准测试
```

### 主要模块

| 模块 | 功能 | 关键类型 |
|------|------|----------|
| `cli` | 命令行参数解析 | `Cli`, `Commands` |
| `tui` | 终端界面 | `App`, `Focus` |
| `api` | HTTP API | `AppState`, 各种 handler |
| `storage` | 数据存储 | `Database`, `Command` |
| `search` | 全文搜索 | `SearchEngine`, `SearchResult` |
| `learn` | 学习命令 | `get_help_output`, `parse_help_content` |
| `update` | 数据更新 | `parse_tldr_archive` |
| `config` | 配置管理 | `AppConfig` |

---

## 核心代码阅读

### 入口点: main.rs

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();           // 解析命令行参数
    let config = AppConfig::load_default();  // 加载配置
    
    match cli.command {
        Some(Commands::Serve { port, bind }) => {
            run_server(&bind, port, config).await
        }
        None => {
            if let Some(query) = cli.query {
                run_query(&query, &cli.lang, &config).await
            } else {
                run_tui(cli.debug, config).await
            }
        }
        // ... 其他命令
    }
}
```

### TUI 模块

**应用状态 (app.rs):**

```rust
pub struct App {
    pub db: Database,                    // 数据库
    pub search: Arc<RwLock<SearchEngine>>,  // 搜索引擎
    pub query: String,                   // 当前搜索词
    pub results: Vec<SearchResult>,      // 搜索结果
    pub selected: usize,                 // 当前选中项
    pub focus: Focus,                    // 当前焦点
    // ...
}
```

**事件循环 (mod.rs):**

```rust
async fn run_app(terminal: &mut Terminal<...>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;  // 渲染 UI
        
        if let Some(event) = poll_event(timeout)? {  // 轮询事件
            match event {
                Event::Key(key) => {
                    match handle_key_event(app, key) {
                        EventResult::Search => app.search().await,
                        EventResult::Quit => break,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

### 搜索模块

```rust
impl SearchEngine {
    pub fn search(&self, query: &str, lang: Option<&str>, limit: usize) 
        -> Result<SearchResponse, SearchError> {
        let start = std::time::Instant::now();
        
        // 分词并转义特殊字符
        let tokenized = self.tokenize_query(query);
        
        // 执行搜索
        let searcher = self.reader.searcher();
        let query = self.parser.parse_query(&tokenized)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        // 构建结果
        let results = top_docs.iter().map(|(score, addr)| {
            let doc = searcher.doc(*addr)?;
            SearchResult { name, description, score, ... }
        }).collect();
        
        Ok(SearchResponse {
            total: results.len(),
            results,
            took_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

---

## 常见任务

### 添加新的命令行选项

1. 编辑 `src/cli/mod.rs`:

```rust
#[derive(Parser)]
pub struct Cli {
    // 添加新选项
    #[arg(short, long)]
    pub new_option: bool,
}
```

2. 在 `main.rs` 中使用:

```rust
if cli.new_option {
    // 处理新选项
}
```

### 添加新的 API 端点

1. 创建处理函数 (`src/api/your_module.rs`):

```rust
use axum::Json;

#[derive(Serialize, ToSchema)]
pub struct YourResponse {
    pub data: String,
}

#[utoipa::path(
    get,
    path = "/api/your-endpoint",
    responses((status = 200, body = YourResponse)),
    tag = "YourTag"
)]
pub async fn your_handler() -> Json<YourResponse> {
    Json(YourResponse { data: "hello".into() })
}
```

2. 在 `src/api/mod.rs` 中注册:

```rust
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/your-endpoint", get(your_module::your_handler))
        // ...
}
```

### 修改配置选项

1. 在 `src/config.rs` 添加字段:

```rust
pub struct YourConfig {
    pub new_option: String,
}

impl Default for YourConfig {
    fn default() -> Self {
        Self {
            new_option: "default".to_string(),
        }
    }
}
```

2. 更新 `rtfm.example.toml`

---

## 调试技巧

### 打印调试

```rust
// 简单打印
println!("value: {}", some_value);

// 调试格式（显示更多信息）
println!("value: {:?}", some_value);  // 需要 #[derive(Debug)]

// 带变量名的调试
dbg!(some_value);  // 打印: [src/main.rs:10] some_value = ...
```

### 使用日志

```rust
use tracing::{info, debug, warn, error};

info!("Processing command: {}", name);
debug!("Search results: {:?}", results);
warn!("Config not found, using defaults");
error!("Failed to open database: {}", e);
```

运行时启用:

```bash
RUST_LOG=debug cargo run
RUST_LOG=rtfm=trace cargo run  # 只看 rtfm 的详细日志
```

### TUI 调试模式

```bash
cargo run -- --debug
# 然后按 F12 显示日志面板
```

---

## 常见问题

### Q: 编译错误 "borrowed value does not live long enough"

**原因:** 借用的值在被使用时已经失效

**解决:** 
- 使用 `.clone()` 创建副本
- 调整代码结构确保值存活足够长
- 使用 `Arc` 进行引用计数共享

### Q: 编译错误 "cannot move out of borrowed content"

**原因:** 尝试从借用中取得所有权

**解决:**
- 使用 `.clone()` 复制
- 改用借用 (`&T`) 而非所有权 (`T`)

### Q: 编译错误 "trait bound not satisfied"

**原因:** 类型没有实现必要的 trait

**解决:**
```rust
// 添加 derive 宏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyType { ... }
```

### Q: cargo build 很慢

**解决:**
```bash
# 增量编译（默认启用）
# 第一次会慢，之后会快

# 使用 sccache 缓存编译结果
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Q: 如何理解生命周期标注 `'a`?

```rust
// 'a 表示 output 的生命周期不能超过 input
fn first_word<'a>(input: &'a str) -> &'a str {
    input.split(' ').next().unwrap_or("")
}
```

简单规则：大多数情况编译器会自动推断，只在编译器报错时添加。

---

## 学习资源

- [The Rust Book](https://doc.rust-lang.org/book/) - 官方教程
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - 示例学习
- [Rustlings](https://github.com/rust-lang/rustlings) - 交互式练习
- [Rust语言圣经](https://course.rs/) - 中文教程

---

## 下一步

1. 运行 `cargo test` 确保测试通过
2. 阅读 `src/main.rs` 理解程序入口
3. 尝试修改一个小功能
4. 提交 PR！

欢迎提问和贡献！
