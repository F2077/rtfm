use std::path::Path;

use redb::{Database as RedbDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

const COMMANDS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("commands");
const METADATA_TABLE: TableDefinition<&str, &str> = TableDefinition::new("metadata");

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("Storage error: {0}")]
    Storage(#[from] redb::StorageError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Command {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command category (e.g., common, linux, windows)
    pub category: String,
    /// Target platform
    pub platform: String,
    /// Language code (e.g., en, zh)
    pub lang: String,
    /// Usage examples
    pub examples: Vec<Example>,
    /// Raw help content
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Example {
    /// Example description
    pub description: String,
    /// Example code
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Metadata {
    /// Data version
    pub version: String,
    /// Total command count
    pub command_count: usize,
    /// Last update timestamp
    pub last_update: String,
    /// Available languages
    pub languages: Vec<String>,
}

pub struct Database {
    db: RedbDatabase,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let db = RedbDatabase::create(path)?;

        // 初始化表
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(COMMANDS_TABLE)?;
            let _ = write_txn.open_table(METADATA_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    pub fn get_command(&self, name: &str, lang: &str) -> Result<Option<Command>, StorageError> {
        let key = format!("{}:{}", lang, name);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(COMMANDS_TABLE)?;

        match table.get(key.as_str())? {
            Some(data) => {
                let cmd: Command = serde_json::from_slice(data.value())?;
                Ok(Some(cmd))
            }
            None => Ok(None),
        }
    }

    pub fn save_command(&self, cmd: &Command) -> Result<(), StorageError> {
        let key = format!("{}:{}", cmd.lang, cmd.name);
        let data = serde_json::to_vec(cmd)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(COMMANDS_TABLE)?;
            table.insert(key.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn save_commands(&self, commands: &[Command]) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(COMMANDS_TABLE)?;
            for cmd in commands {
                let key = format!("{}:{}", cmd.lang, cmd.name);
                let data = serde_json::to_vec(cmd)?;
                table.insert(key.as_str(), data.as_slice())?;
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn get_all_commands(&self, lang: &str) -> Result<Vec<Command>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(COMMANDS_TABLE)?;

        let prefix = format!("{}:", lang);
        let mut commands = Vec::new();

        for entry in table.iter()? {
            let (key, value) = entry?;
            if key.value().starts_with(&prefix) {
                let cmd: Command = serde_json::from_slice(value.value())?;
                commands.push(cmd);
            }
        }

        Ok(commands)
    }

    pub fn get_metadata(&self) -> Result<Option<Metadata>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(METADATA_TABLE)?;

        match table.get("metadata")? {
            Some(data) => {
                let meta: Metadata = serde_json::from_str(data.value())?;
                Ok(Some(meta))
            }
            None => Ok(None),
        }
    }

    pub fn save_metadata(&self, meta: &Metadata) -> Result<(), StorageError> {
        let data = serde_json::to_string(meta)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(METADATA_TABLE)?;
            table.insert("metadata", data.as_str())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn clear_commands(&self) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write()?;
        {
            // 删除并重新创建表
            write_txn.delete_table(COMMANDS_TABLE)?;
            let _ = write_txn.open_table(COMMANDS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn count_commands(&self) -> Result<usize, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(COMMANDS_TABLE)?;
        Ok(table.len()? as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_command(name: &str, lang: &str) -> Command {
        Command {
            name: name.to_string(),
            description: format!("{} command", name),
            category: "common".to_string(),
            platform: "common".to_string(),
            lang: lang.to_string(),
            examples: vec![
                Example {
                    description: "Example usage".to_string(),
                    code: format!("{} --help", name),
                },
            ],
            content: format!("{} help content", name),
        }
    }

    #[test]
    fn test_database_create() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path);
        assert!(db.is_ok());
    }

    #[test]
    fn test_save_and_get_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        let cmd = create_test_command("docker", "en");
        db.save_command(&cmd).unwrap();

        let retrieved = db.get_command("docker", "en").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "docker");
        assert_eq!(retrieved.lang, "en");
    }

    #[test]
    fn test_save_commands_batch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        let commands = vec![
            create_test_command("docker", "en"),
            create_test_command("tar", "en"),
            create_test_command("git", "en"),
        ];

        db.save_commands(&commands).unwrap();
        assert_eq!(db.count_commands().unwrap(), 3);
    }

    #[test]
    fn test_get_nonexistent_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        let result = db.get_command("nonexistent", "en").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear_commands() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        let commands = vec![
            create_test_command("docker", "en"),
            create_test_command("tar", "en"),
        ];
        db.save_commands(&commands).unwrap();
        assert_eq!(db.count_commands().unwrap(), 2);

        db.clear_commands().unwrap();
        assert_eq!(db.count_commands().unwrap(), 0);
    }

    #[test]
    fn test_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        let meta = Metadata {
            version: "1.0.0".to_string(),
            command_count: 100,
            last_update: "2024-01-01".to_string(),
            languages: vec!["en".to_string(), "zh".to_string()],
        };

        db.save_metadata(&meta).unwrap();

        let retrieved = db.get_metadata().unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.version, "1.0.0");
        assert_eq!(retrieved.command_count, 100);
    }

    #[test]
    fn test_multilang_commands() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::open(&db_path).unwrap();

        db.save_command(&create_test_command("docker", "en")).unwrap();
        db.save_command(&create_test_command("docker", "zh")).unwrap();

        let en_cmd = db.get_command("docker", "en").unwrap();
        let zh_cmd = db.get_command("docker", "zh").unwrap();

        assert!(en_cmd.is_some());
        assert!(zh_cmd.is_some());
        assert_eq!(en_cmd.unwrap().lang, "en");
        assert_eq!(zh_cmd.unwrap().lang, "zh");
    }
}
