use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Serialize, TS)]
#[ts(export)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("branch moved — retry with updated revision")]
    BranchMoved,

    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    pub fn http_status(&self) -> u16 {
        match self {
            ApiError::NotFound(_) => 404,
            ApiError::BranchMoved => 409,
            ApiError::Internal(_) => 500,
        }
    }
}

// ── Shared DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BranchInfo {
    pub name: String,
    pub head_revision: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RevisionInfo {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub content_hash: String,
    pub asset_kind: AssetKind,
    pub lock_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AssetKind {
    Text,
    Markdown,
    Image,
    Model3d,
    Blend,
    Uasset,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StatusEntry {
    pub path: String,
    pub state: FileState,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FileState {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

// ── Health ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
