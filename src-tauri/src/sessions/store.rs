//! JSON-backed session history with validation and atomic writes.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;
use magi_core::{
    MAX_STORED_SESSIONS, SESSION_SCHEMA_VERSION, SessionDraft, SessionRecord, SessionSource,
    SessionSummary,
};
use serde::{Deserialize, Serialize};

use crate::credentials::set_private_permissions;

#[derive(Debug, Serialize, Deserialize)]
struct SessionsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    sessions: Vec<SessionRecord>,
}

#[derive(Default)]
struct Inner {
    sessions: Vec<SessionRecord>,
    /// Set when the on-disk file was written by a newer schema. Writes are
    /// refused so that newer data is never overwritten.
    blocked: bool,
}

pub struct SessionStore {
    path: PathBuf,
    inner: Mutex<Inner>,
}

impl SessionStore {
    pub fn load(data_dir: PathBuf) -> Self {
        let path = data_dir.join("sessions.json");

        let inner = match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<SessionsFile>(&raw) {
                Ok(file) if file.schema_version > SESSION_SCHEMA_VERSION => {
                    tracing::error!(
                        path = %path.display(),
                        found = file.schema_version,
                        supported = SESSION_SCHEMA_VERSION,
                        "history uses a newer schema; writes disabled"
                    );
                    Inner {
                        sessions: Vec::new(),
                        blocked: true,
                    }
                }
                Ok(file) => {
                    let mut sessions = file.sessions;
                    sort_sessions(&mut sessions);
                    sessions.truncate(MAX_STORED_SESSIONS);
                    Inner {
                        sessions,
                        blocked: false,
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        %error,
                        "invalid history file; starting empty"
                    );
                    backup_corrupt(&path);
                    Inner::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Inner::default(),
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    %error,
                    "cannot read history; starting empty"
                );
                Inner::default()
            }
        };

        Self {
            path,
            inner: Mutex::new(inner),
        }
    }

    pub fn list(&self) -> Vec<SessionSummary> {
        let inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner
            .sessions
            .iter()
            .map(SessionSummary::from_record)
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<SessionRecord> {
        let inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner
            .sessions
            .iter()
            .find(|record| record.id == id)
            .cloned()
    }

    pub fn save(
        &self,
        mut draft: SessionDraft,
        source: SessionSource,
    ) -> Result<SessionSummary, String> {
        draft.recompute().map_err(|error| error.to_string())?;

        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if inner.blocked {
            return Err("历史文件由更新版本的 MAGI 写入，已进入只读保护".into());
        }

        let now = Utc::now().to_rfc3339();
        let created_at = inner
            .sessions
            .iter()
            .find(|record| record.id == draft.id)
            .map(|record| record.created_at.clone())
            .unwrap_or_else(|| now.clone());

        let record = SessionRecord::from_draft(draft, source, created_at, now);
        let summary = SessionSummary::from_record(&record);

        // Build the next state and persist it before publishing it in memory.
        let mut sessions = inner.sessions.clone();
        if let Some(existing) = sessions.iter_mut().find(|item| item.id == record.id) {
            *existing = record;
        } else {
            sessions.push(record);
        }
        sort_sessions(&mut sessions);
        sessions.truncate(MAX_STORED_SESSIONS);

        persist(&self.path, &sessions)?;
        inner.sessions = sessions;

        Ok(summary)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if inner.blocked {
            return Err("历史文件由更新版本的 MAGI 写入，已进入只读保护".into());
        }

        let mut sessions = inner.sessions.clone();
        sessions.retain(|record| record.id != id);

        persist(&self.path, &sessions)?;
        inner.sessions = sessions;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if inner.blocked {
            return Err("历史文件由更新版本的 MAGI 写入，已进入只读保护".into());
        }

        persist(&self.path, &[])?;
        inner.sessions.clear();
        Ok(())
    }
}

fn sort_sessions(sessions: &mut [SessionRecord]) {
    sessions.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.id.cmp(&left.id))
    });
}

fn persist(path: &Path, sessions: &[SessionRecord]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建历史目录失败: {error}"))?;
    }

    let file = SessionsFile {
        schema_version: SESSION_SCHEMA_VERSION,
        sessions: sessions.to_vec(),
    };
    let raw =
        serde_json::to_string_pretty(&file).map_err(|error| format!("序列化历史失败: {error}"))?;

    let temp = path.with_extension("json.tmp");
    {
        let mut handle =
            fs::File::create(&temp).map_err(|error| format!("写入历史失败: {error}"))?;
        handle
            .write_all(raw.as_bytes())
            .map_err(|error| format!("写入历史失败: {error}"))?;
        handle
            .sync_all()
            .map_err(|error| format!("同步历史失败: {error}"))?;
    }
    set_private_permissions(&temp)?;

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("替换历史失败: {error}"))?;
    }

    fs::rename(&temp, path).map_err(|error| format!("保存历史失败: {error}"))
}

fn backup_corrupt(path: &Path) {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let backup = path.with_file_name(format!("sessions.corrupt-{timestamp}.json"));

    match fs::rename(path, &backup) {
        Ok(()) => tracing::warn!(path = %backup.display(), "history backup written"),
        Err(error) => tracing::warn!(%error, "could not back up corrupt history"),
    }
}
