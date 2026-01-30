use std::path::Path;

use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, IndexWriter, TantivyDocument};
use thiserror::Error;
use utoipa::ToSchema;

use crate::storage::Command;

static JIEBA: Lazy<Jieba> = Lazy::new(Jieba::new);

#[derive(Error, Debug)]
pub enum SearchError {
  #[error("Tantivy error: {0}")]
  Tantivy(#[from] tantivy::TantivyError),
  #[error("Query parse error: {0}")]
  QueryParser(#[from] tantivy::query::QueryParserError),
  #[error("Open directory error: {0}")]
  OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
  /// Command name
  pub name: String,
  /// Command description
  pub description: String,
  /// Command category
  pub category: String,
  /// Language code
  pub lang: String,
  /// Search relevance score
  pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
  /// Total results count
  pub total: usize,
  /// Search results
  pub results: Vec<SearchResult>,
  /// Query execution time in milliseconds
  pub took_ms: u64,
}

pub struct SearchEngine {
  index: Index,
  reader: IndexReader,
  #[allow(dead_code)]
  schema: Schema,
  name_field: Field,
  description_field: Field,
  content_field: Field,
  category_field: Field,
  lang_field: Field,
}

impl SearchEngine {
  pub fn open(path: &Path) -> Result<Self, SearchError> {
    std::fs::create_dir_all(path)?;

    // 构建 Schema
    let mut schema_builder = Schema::builder();
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);
    let description_field = schema_builder.add_text_field("description", TEXT | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT);
    let category_field = schema_builder.add_text_field("category", TEXT | STORED);
    let lang_field = schema_builder.add_text_field("lang", TEXT | STORED);
    let schema = schema_builder.build();

    // 打开或创建索引
    let index = if path.join("meta.json").exists() {
      Index::open_in_dir(path)?
    } else {
      Index::create_in_dir(path, schema.clone())?
    };

    // 注册自定义分词器（简单分词 + 小写）
    let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
      .filter(LowerCaser)
      .build();
    index.tokenizers().register("default", tokenizer);

    let reader = index.reader()?;

    Ok(Self {
      index,
      reader,
      schema,
      name_field,
      description_field,
      content_field,
      category_field,
      lang_field,
    })
  }

  pub fn index_commands(&mut self, commands: &[Command]) -> Result<(), SearchError> {
    let mut writer: IndexWriter = self.index.writer(50_000_000)?;

    // 清空现有索引
    writer.delete_all_documents()?;

    for cmd in commands {
      let mut doc = TantivyDocument::default();

      // 对 name 和 description 也进行 jieba 分词，保持与查询时一致
      let tokenized_name = self.tokenize_chinese(&cmd.name);
      let tokenized_description = self.tokenize_chinese(&cmd.description);
      doc.add_text(self.name_field, &tokenized_name);
      doc.add_text(self.description_field, &tokenized_description);

      // 对内容进行 jieba 分词后存入
      let tokenized_content = self.tokenize_chinese(&cmd.content);
      doc.add_text(self.content_field, &tokenized_content);

      doc.add_text(self.category_field, &cmd.category);
      doc.add_text(self.lang_field, &cmd.lang);

      writer.add_document(doc)?;
    }

    writer.commit()?;
    self.reader.reload()?;

    Ok(())
  }

  /// 增量索引单个命令
  pub fn index_single_command(&mut self, cmd: &Command) -> Result<(), SearchError> {
    let mut writer: IndexWriter = self.index.writer(50_000_000)?;

    let mut doc = TantivyDocument::default();

    // 对 name 和 description 也进行 jieba 分词，保持与查询时一致
    let tokenized_name = self.tokenize_chinese(&cmd.name);
    let tokenized_description = self.tokenize_chinese(&cmd.description);
    doc.add_text(self.name_field, &tokenized_name);
    doc.add_text(self.description_field, &tokenized_description);

    let tokenized_content = self.tokenize_chinese(&cmd.content);
    doc.add_text(self.content_field, &tokenized_content);

    doc.add_text(self.category_field, &cmd.category);
    doc.add_text(self.lang_field, &cmd.lang);

    writer.add_document(doc)?;
    writer.commit()?;
    self.reader.reload()?;

    Ok(())
  }

  pub fn search(
    &self,
    query: &str,
    lang: Option<&str>,
    limit: usize,
  ) -> Result<SearchResponse, SearchError> {
    let start = std::time::Instant::now();

    let searcher = self.reader.searcher();

    // 对查询进行分词并转义特殊字符
    let tokenized_query = self.tokenize_and_escape(query);

    // 构建查询
    let query_parser = QueryParser::for_index(
      &self.index,
      vec![self.name_field, self.description_field, self.content_field],
    );

    // 如果指定了语言，添加语言过滤
    let query_str = if let Some(l) = lang {
      format!("({}) AND lang:{}", tokenized_query, l)
    } else {
      tokenized_query
    };

    let parsed_query = query_parser.parse_query(&query_str)?;
    let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
      let doc: TantivyDocument = searcher.doc(doc_address)?;

      let name = doc
        .get_first(self.name_field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

      let description = doc
        .get_first(self.description_field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

      let category = doc
        .get_first(self.category_field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

      let lang = doc
        .get_first(self.lang_field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

      results.push(SearchResult {
        name,
        description,
        category,
        lang,
        score,
      });
    }

    let took_ms = start.elapsed().as_millis() as u64;

    Ok(SearchResponse {
      total: results.len(),
      results,
      took_ms,
    })
  }

  /// 分词并转义 Tantivy 特殊字符
  fn tokenize_and_escape(&self, text: &str) -> String {
    // 先用 jieba 分词
    let tokens = JIEBA.cut(text, true);
    // 转义每个 token 中的特殊字符
    tokens
      .into_iter()
      .map(Self::escape_special_chars)
      .collect::<Vec<_>>()
      .join(" ")
  }

  /// 转义 Tantivy 查询语法中的特殊字符
  fn escape_special_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
      // Tantivy 特殊字符: + - && || ! ( ) { } [ ] ^ " ~ * ? : \ /
      if matches!(
        c,
        '+'
          | '-'
          | '!'
          | '('
          | ')'
          | '{'
          | '}'
          | '['
          | ']'
          | '^'
          | '"'
          | '~'
          | '*'
          | '?'
          | ':'
          | '\\'
          | '/'
      ) {
        result.push('\\');
      }
      result.push(c);
    }
    result
  }

  fn tokenize_chinese(&self, text: &str) -> String {
    let tokens = JIEBA.cut(text, true);
    tokens.join(" ")
  }

  pub fn reload(&mut self) -> Result<(), SearchError> {
    self.reader.reload()?;
    Ok(())
  }

  /// 清空索引（用于重置）
  pub fn clear(&mut self) -> Result<(), SearchError> {
    let mut writer: IndexWriter = self.index.writer(50_000_000)?;
    writer.delete_all_documents()?;
    writer.commit()?;
    self.reader.reload()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_escape_special_chars() {
    assert_eq!(SearchEngine::escape_special_chars("docker"), "docker");
    assert_eq!(SearchEngine::escape_special_chars("-a"), "\\-a");
    assert_eq!(SearchEngine::escape_special_chars("ps -a"), "ps \\-a");
    assert_eq!(SearchEngine::escape_special_chars("foo+bar"), "foo\\+bar");
    assert_eq!(SearchEngine::escape_special_chars("a*b?c"), "a\\*b\\?c");
    assert_eq!(
      SearchEngine::escape_special_chars("path/to/file"),
      "path\\/to\\/file"
    );
  }

  #[test]
  fn test_tokenize_chinese() {
    let jieba = Jieba::new();
    let tokens = jieba.cut("复制文件", true);
    assert!(tokens.len() >= 2);
  }

  #[test]
  fn test_search_engine_create() {
    let temp_dir = tempfile::tempdir().unwrap();
    let engine = SearchEngine::open(temp_dir.path());
    assert!(engine.is_ok());
  }

  #[test]
  fn test_index_and_search() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut engine = SearchEngine::open(temp_dir.path()).unwrap();

    let commands = vec![
      Command {
        name: "docker".to_string(),
        description: "Manage Docker containers".to_string(),
        category: "common".to_string(),
        platform: "common".to_string(),
        lang: "en".to_string(),
        examples: vec![],
        content: "docker ps -a".to_string(),
      },
      Command {
        name: "tar".to_string(),
        description: "Archive files".to_string(),
        category: "common".to_string(),
        platform: "common".to_string(),
        lang: "en".to_string(),
        examples: vec![],
        content: "tar -xvf file.tar".to_string(),
      },
    ];

    engine.index_commands(&commands).unwrap();

    // 测试搜索
    let results = engine.search("docker", None, 10).unwrap();
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].name, "docker");

    // 测试特殊字符
    let results = engine.search("ps -a", None, 10).unwrap();
    assert!(!results.results.is_empty());
  }
}
