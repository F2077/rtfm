use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;

// 注意：基准测试需要在有数据的环境下运行
// 运行: cargo bench

fn bench_search(c: &mut Criterion) {
    // 尝试打开已有的索引
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rtfm");
    
    let index_path = data_dir.join("index");
    
    if !index_path.exists() {
        eprintln!("Skipping benchmark: index not found at {:?}", index_path);
        eprintln!("Run 'rtfm update' first to download data.");
        return;
    }

    // 这里我们只测试字符串处理性能
    c.bench_function("escape_special_chars", |b| {
        b.iter(|| {
            let input = black_box("docker ps -a --format '{{.Names}}'");
            escape_special_chars(input)
        })
    });

    c.bench_function("tokenize_chinese", |b| {
        let jieba = jieba_rs::Jieba::new();
        b.iter(|| {
            let input = black_box("复制文件到容器");
            let tokens = jieba.cut(input, true);
            tokens.join(" ")
        })
    });
}

fn escape_special_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if matches!(c, '+' | '-' | '!' | '(' | ')' | '{' | '}' | '[' | ']' 
                     | '^' | '"' | '~' | '*' | '?' | ':' | '\\' | '/') {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
