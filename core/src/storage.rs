//! Encrypted, account-scoped local persistence shared by native clients.
//!
//! SQLCipher is mandatory. Opening fails when the linked SQLite library does
//! not expose `PRAGMA cipher_version`; plaintext fallback is never allowed.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use zeroize::Zeroize;

pub const CURRENT_CATALOG_SCHEMA: i32 = 1;
pub const CURRENT_CONVERSATION_SCHEMA: i32 = 1;
pub const CURRENT_SEARCH_SCHEMA: i32 = 1;
pub const CURRENT_JOBS_SCHEMA: i32 = 1;
pub const CODEC_JSON_UTF8: i32 = 1;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database key must contain exactly 32 bytes")]
    InvalidKey,
    #[error("SQLCipher is unavailable; refusing to open an unencrypted database")]
    CipherUnavailable,
    #[error("database lock is poisoned")]
    LockPoisoned,
    #[error("all cached conversation handles are currently in use")]
    HandleLimitReached,
    #[error("invalid database path")]
    InvalidPath,
    #[error("migration {version} ({name}) checksum does not match the applied migration")]
    MigrationChecksumMismatch { version: i32, name: &'static str },
    #[error("database schema version {found} is newer than supported version {supported}")]
    UnsupportedSchema { found: i32, supported: i32 },
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ProfileType {
    User = 1,
    Group = 2,
    Channel = 3,
    OfficialAccount = 4,
    Bot = 5,
    Community = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum CoverType {
    Image = 1,
    Video = 2,
    AnimatedImage = 3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodedData {
    pub codec: i32,
    pub version: i32,
    pub bytes: Vec<u8>,
}

impl EncodedData {
    pub fn json<T: Serialize>(version: i32, value: &T) -> Result<Self> {
        Ok(Self {
            codec: CODEC_JSON_UTF8,
            version,
            bytes: serde_json::to_vec(value)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProfileRecord {
    pub profile_id: i64,
    pub profile_type: ProfileType,
    pub display_name: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_url: Option<String>,
    pub cover_type: Option<CoverType>,
    pub profile_version: i64,
    pub verification_state: i32,
    pub relationship_state: i32,
    pub updated_at_ms: i64,
    pub metadata: EncodedData,
}

#[derive(Clone, Debug)]
pub struct ConversationSummaryRecord {
    pub conversation_id: i64,
    pub conversation_type: i32,
    pub peer_id: i64,
    pub peer_type: ProfileType,
    pub display_name: String,
    pub display_avatar_url: Option<String>,
    pub last_message_id: Option<i64>,
    pub last_message_sender_id: Option<i64>,
    pub last_message_sender_type: Option<ProfileType>,
    pub last_message_sender_name: Option<String>,
    pub last_message_type: Option<i32>,
    pub last_message_text: Option<String>,
    pub last_message_at_ms: Option<i64>,
    pub unread_count: i64,
    pub metadata: EncodedData,
}

/// Owns the always-open account catalog and a bounded LRU of conversation DBs.
pub struct AccountStore {
    root: PathBuf,
    key: [u8; 32],
    catalog: Mutex<Connection>,
    search: Mutex<Connection>,
    jobs: Mutex<Connection>,
    conversations: Mutex<ConversationHandles>,
}

struct ConversationHandles {
    max_open: usize,
    handles: HashMap<i64, Arc<Mutex<Connection>>>,
    lru: VecDeque<i64>,
}

impl AccountStore {
    pub fn open(root: impl AsRef<Path>, key: &[u8], max_open: usize) -> Result<Self> {
        let key: [u8; 32] = key.try_into().map_err(|_| StorageError::InvalidKey)?;
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("conversations"))?;
        let mut catalog = open_encrypted(&root.join("catalog.db"), &key)?;
        migrate_catalog(&mut catalog)?;
        let mut search = open_encrypted(&root.join("search.db"), &key)?;
        migrate_search(&mut search)?;
        let mut jobs = open_encrypted(&root.join("jobs.db"), &key)?;
        migrate_jobs(&mut jobs)?;
        Ok(Self {
            root,
            key,
            catalog: Mutex::new(catalog),
            search: Mutex::new(search),
            jobs: Mutex::new(jobs),
            conversations: Mutex::new(ConversationHandles {
                max_open: max_open.clamp(1, 32),
                handles: HashMap::new(),
                lru: VecDeque::new(),
            }),
        })
    }

    pub fn upsert_profile(&self, profile: &ProfileRecord) -> Result<bool> {
        let db = self.catalog.lock().map_err(|_| StorageError::LockPoisoned)?;
        let changed = db.execute(
            "INSERT INTO profiles(profile_id,profile_type,display_name,username,avatar_url,cover_url,cover_type,profile_version,verification_state,relationship_state,updated_at_ms,metadata_codec,metadata_version,metadata)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(profile_type,profile_id) DO UPDATE SET
               display_name=excluded.display_name,username=excluded.username,avatar_url=excluded.avatar_url,
               cover_url=excluded.cover_url,cover_type=excluded.cover_type,profile_version=excluded.profile_version,
               verification_state=excluded.verification_state,relationship_state=excluded.relationship_state,
               updated_at_ms=excluded.updated_at_ms,metadata_codec=excluded.metadata_codec,
               metadata_version=excluded.metadata_version,metadata=excluded.metadata
             WHERE excluded.profile_version > profiles.profile_version",
            params![
                profile.profile_id, profile.profile_type as i32, profile.display_name, profile.username,
                profile.avatar_url, profile.cover_url, profile.cover_type.map(|v| v as i32),
                profile.profile_version, profile.verification_state, profile.relationship_state,
                profile.updated_at_ms, profile.metadata.codec, profile.metadata.version, profile.metadata.bytes,
            ],
        )?;
        Ok(changed > 0)
    }

    pub fn upsert_conversation(&self, row: &ConversationSummaryRecord) -> Result<()> {
        let db = self.catalog.lock().map_err(|_| StorageError::LockPoisoned)?;
        db.execute(
            "INSERT INTO conversations(conversation_id,conversation_type,peer_id,peer_type,display_name,display_avatar_url,last_message_id,last_message_sender_id,last_message_sender_type,last_message_sender_name,last_message_type,last_message_text,last_message_at_ms,unread_count,metadata_codec,metadata_version,metadata)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
             ON CONFLICT(conversation_id) DO UPDATE SET
               conversation_type=excluded.conversation_type,peer_id=excluded.peer_id,peer_type=excluded.peer_type,
               display_name=excluded.display_name,display_avatar_url=excluded.display_avatar_url,
               last_message_id=excluded.last_message_id,last_message_sender_id=excluded.last_message_sender_id,
               last_message_sender_type=excluded.last_message_sender_type,last_message_sender_name=excluded.last_message_sender_name,
               last_message_type=excluded.last_message_type,last_message_text=excluded.last_message_text,
               last_message_at_ms=excluded.last_message_at_ms,unread_count=excluded.unread_count,
               metadata_codec=excluded.metadata_codec,metadata_version=excluded.metadata_version,metadata=excluded.metadata",
            params![
                row.conversation_id,row.conversation_type,row.peer_id,row.peer_type as i32,row.display_name,
                row.display_avatar_url,row.last_message_id,row.last_message_sender_id,
                row.last_message_sender_type.map(|v| v as i32),row.last_message_sender_name,
                row.last_message_type,row.last_message_text,row.last_message_at_ms,row.unread_count,
                row.metadata.codec,row.metadata.version,row.metadata.bytes,
            ],
        )?;
        Ok(())
    }

    /// Lazily creates and returns an encrypted DB for one conversation.
    pub fn conversation(&self, conversation_id: i64) -> Result<Arc<Mutex<Connection>>> {
        if conversation_id <= 0 {
            return Err(StorageError::InvalidPath);
        }
        let mut cache = self.conversations.lock().map_err(|_| StorageError::LockPoisoned)?;
        if let Some(existing) = cache.handles.get(&conversation_id).cloned() {
            touch(&mut cache.lru, conversation_id);
            return Ok(existing);
        }
        while cache.handles.len() >= cache.max_open {
            let evictable = cache.lru.iter().position(|id| {
                cache
                    .handles
                    .get(id)
                    .is_some_and(|handle| Arc::strong_count(handle) == 1)
            });
            let Some(index) = evictable else { break };
            let Some(oldest) = cache.lru.remove(index) else {
                break;
            };
            cache.handles.remove(&oldest);
        }
        if cache.handles.len() >= cache.max_open {
            return Err(StorageError::HandleLimitReached);
        }
        let path = self.conversation_path(conversation_id);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        let mut db = open_encrypted(&path, &self.key)?;
        migrate_conversation(&mut db)?;
        let handle = Arc::new(Mutex::new(db));
        cache.handles.insert(conversation_id, Arc::clone(&handle));
        touch(&mut cache.lru, conversation_id);
        Ok(handle)
    }

    pub fn open_handle_count(&self) -> Result<usize> {
        Ok(self.conversations.lock().map_err(|_| StorageError::LockPoisoned)?.handles.len())
    }

    pub fn index_message(
        &self,
        conversation_id: i64,
        message_id: i64,
        sender_id: i64,
        sent_at_ms: i64,
        normalized_text: &str,
        content_type: i32,
    ) -> Result<()> {
        let db = self.search.lock().map_err(|_| StorageError::LockPoisoned)?;
        db.execute(
            "INSERT INTO message_search(conversation_id,message_id,sender_id,sent_at_ms,normalized_text,content_type)
             VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(conversation_id,message_id) DO UPDATE SET sender_id=excluded.sender_id,
             sent_at_ms=excluded.sent_at_ms,normalized_text=excluded.normalized_text,content_type=excluded.content_type",
            params![conversation_id,message_id,sender_id,sent_at_ms,normalized_text,content_type],
        )?;
        Ok(())
    }

    pub fn enqueue_job(&self, job_id: i64, job_type: i32, created_at_ms: i64, payload: &EncodedData) -> Result<()> {
        let db = self.jobs.lock().map_err(|_| StorageError::LockPoisoned)?;
        db.execute(
            "INSERT OR IGNORE INTO jobs(job_id,job_type,state,attempt_count,created_at_ms,payload_codec,payload_version,payload)
             VALUES(?1,?2,0,0,?3,?4,?5,?6)",
            params![job_id,job_type,created_at_ms,payload.codec,payload.version,payload.bytes],
        )?;
        Ok(())
    }

    fn conversation_path(&self, conversation_id: i64) -> PathBuf {
        let name = format!("{conversation_id:016x}");
        self.root.join("conversations").join(&name[..2]).join(format!("{name}.db"))
    }
}

impl Drop for AccountStore {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

fn touch(lru: &mut VecDeque<i64>, id: i64) {
    if let Some(index) = lru.iter().position(|value| *value == id) { lru.remove(index); }
    lru.push_back(id);
}

fn open_encrypted(path: &Path, key: &[u8; 32]) -> Result<Connection> {
    let db = Connection::open(path)?;
    let hex_key = hex::encode(key);
    db.pragma_update(None, "key", format!("x'{hex_key}'"))?;
    let cipher: Option<String> = db.query_row("PRAGMA cipher_version", [], |row| row.get(0)).optional()?;
    if cipher.as_deref().is_none_or(|version| version.is_empty()) {
        return Err(StorageError::CipherUnavailable);
    }
    db.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
    db.pragma_update(None, "cipher_memory_security", "ON")?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "NORMAL")?;
    db.pragma_update(None, "foreign_keys", "ON")?;
    db.pragma_update(None, "busy_timeout", 3000)?;
    Ok(db)
}

fn migrate_catalog(db: &mut Connection) -> Result<()> {
    apply_migrations(db, CATALOG_MIGRATIONS)
}

fn migrate_conversation(db: &mut Connection) -> Result<()> {
    apply_migrations(db, CONVERSATION_MIGRATIONS)
}

fn migrate_search(db: &mut Connection) -> Result<()> {
    apply_migrations(db, SEARCH_MIGRATIONS)
}

fn migrate_jobs(db: &mut Connection) -> Result<()> {
    apply_migrations(db, JOBS_MIGRATIONS)
}

#[derive(Clone, Copy)]
struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

const CATALOG_MIGRATIONS: &[Migration] = &[Migration {
    version: CURRENT_CATALOG_SCHEMA,
    name: "initial",
    sql: include_str!("../migrations/catalog/V001__initial.sql"),
}];
const CONVERSATION_MIGRATIONS: &[Migration] = &[Migration {
    version: CURRENT_CONVERSATION_SCHEMA,
    name: "initial",
    sql: include_str!("../migrations/conversation/V001__initial.sql"),
}];
const SEARCH_MIGRATIONS: &[Migration] = &[Migration {
    version: CURRENT_SEARCH_SCHEMA,
    name: "initial",
    sql: include_str!("../migrations/search/V001__initial.sql"),
}];
const JOBS_MIGRATIONS: &[Migration] = &[Migration {
    version: CURRENT_JOBS_SCHEMA,
    name: "initial",
    sql: include_str!("../migrations/jobs/V001__initial.sql"),
}];

fn apply_migrations(db: &mut Connection, migrations: &[Migration]) -> Result<()> {
    let supported = migrations.last().map_or(0, |migration| migration.version);
    let bootstrap = db.transaction()?;
    bootstrap.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_info(version INTEGER NOT NULL);
         INSERT INTO schema_info(version) SELECT 0 WHERE NOT EXISTS(SELECT 1 FROM schema_info);
         CREATE TABLE IF NOT EXISTS schema_migrations(
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           checksum BLOB NOT NULL,
           applied_at_ms INTEGER NOT NULL
         );",
    )?;
    let found: i32 = bootstrap.query_row("SELECT version FROM schema_info LIMIT 1", [], |row| row.get(0))?;
    if found > supported {
        return Err(StorageError::UnsupportedSchema { found, supported });
    }
    bootstrap.commit()?;

    for migration in migrations {
        let checksum = Sha256::digest(migration.sql.as_bytes()).to_vec();
        let applied: Option<Vec<u8>> = db
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version=?1",
                [migration.version],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(applied) = applied {
            if applied != checksum {
                return Err(StorageError::MigrationChecksumMismatch {
                    version: migration.version,
                    name: migration.name,
                });
            }
            continue;
        }

        let tx = db.transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,checksum,applied_at_ms)
             VALUES(?1,?2,?3,unixepoch() * 1000)",
            params![migration.version, migration.name, checksum],
        )?;
        set_schema_version(&tx, migration.version)?;
        tx.commit()?;
    }
    Ok(())
}

fn set_schema_version(tx: &Transaction<'_>, version: i32) -> Result<()> {
    tx.execute("UPDATE schema_info SET version=?1", [version])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tektalk-{name}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
    }

    #[test]
    fn opens_only_with_the_correct_cipher_key_and_creates_lazy_conversation_db() {
        let root = temp_root("cipher");
        let key = [7_u8; 32];
        let store = AccountStore::open(&root, &key, 2).unwrap();
        assert_eq!(store.open_handle_count().unwrap(), 0);
        let handle = store.conversation(42).unwrap();
        assert_eq!(store.open_handle_count().unwrap(), 1);
        drop(handle);
        drop(store);
        assert!(AccountStore::open(&root, &[8_u8; 32], 2).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn profile_updates_are_monotonic_and_keep_cover_media() {
        let root = temp_root("profile");
        let store = AccountStore::open(&root, &[9_u8; 32], 4).unwrap();
        let metadata = EncodedData::json(1, &serde_json::json!({"bio":"hello"})).unwrap();
        let mut profile = ProfileRecord {
            profile_id: 11, profile_type: ProfileType::User, display_name: "New".into(), username: None,
            avatar_url: Some("https://cdn/avatar.webp".into()), cover_url: Some("https://cdn/cover.mp4".into()),
            cover_type: Some(CoverType::Video), profile_version: 2, verification_state: 0,
            relationship_state: 0, updated_at_ms: 2, metadata,
        };
        assert!(store.upsert_profile(&profile).unwrap());
        profile.profile_version = 1;
        profile.display_name = "Stale".into();
        assert!(!store.upsert_profile(&profile).unwrap());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn records_embedded_migration_version_and_checksum() {
        let root = temp_root("migration-record");
        let store = AccountStore::open(&root, &[10_u8; 32], 4).unwrap();
        let catalog = store.catalog.lock().unwrap();
        let (version, name, checksum_size): (i32, String, usize) = catalog
            .query_row(
                "SELECT version,name,length(checksum) FROM schema_migrations",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(version, CURRENT_CATALOG_SCHEMA);
        assert_eq!(name, "initial");
        assert_eq!(checksum_size, 32);
        drop(catalog);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_changed_applied_migration_checksum() {
        let root = temp_root("migration-checksum");
        let key = [11_u8; 32];
        let store = AccountStore::open(&root, &key, 4).unwrap();
        store
            .catalog
            .lock()
            .unwrap()
            .execute(
                "UPDATE schema_migrations SET checksum=?1 WHERE version=?2",
                params![vec![0_u8; 32], CURRENT_CATALOG_SCHEMA],
            )
            .unwrap();
        drop(store);

        assert!(matches!(
            AccountStore::open(&root, &key, 4),
            Err(StorageError::MigrationChecksumMismatch {
                version: CURRENT_CATALOG_SCHEMA,
                name: "initial"
            })
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
