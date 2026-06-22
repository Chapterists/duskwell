use anyhow::Result;
use toolbelt_types::{
    BranchInfo, FileInfo, FileState, RevisionInfo, StatusEntry, TreeEntry,
};

/// Read-only view of a Lore repository.
///
/// The only crate that provides a real implementation is `lore-gateway` itself
/// (behind the `live-lore` feature flag). Everything else depends on this trait
/// and the in-memory fake so the app is testable without a live Lore instance.
#[async_trait::async_trait]
pub trait LoreRepo: Send + Sync {
    async fn repo_name(&self) -> Result<String>;
    async fn default_branch(&self) -> Result<String>;
    async fn list_branches(&self) -> Result<Vec<BranchInfo>>;

    /// List direct children of `path` at `revision` (lazy — no recursion).
    async fn list_tree(
        &self,
        revision: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>>;

    async fn file_info(&self, revision: &str, path: &str) -> Result<FileInfo>;

    /// Raw bytes for a file (byte-range reads where the backend supports it).
    async fn file_content(
        &self,
        revision: &str,
        path: &str,
        range: Option<(u64, u64)>,
    ) -> Result<Vec<u8>>;

    async fn list_revisions(
        &self,
        branch: &str,
        limit: usize,
        cursor: Option<&str>,
    ) -> Result<Vec<RevisionInfo>>;

    async fn revision_parents(&self, revision: &str) -> Result<Vec<String>>;

    async fn status(&self) -> Result<Vec<StatusEntry>>;

    /// Paths changed between two revisions.
    async fn diff_paths(
        &self,
        from_revision: &str,
        to_revision: &str,
    ) -> Result<Vec<(String, FileState)>>;
}

/// Mutable-store operations (write path, Phase 3).
///
/// Separated from `LoreRepo` so read-only views cannot accidentally call
/// write operations. The CAS token prevents silent overwrites when the branch
/// has moved.
#[async_trait::async_trait]
pub trait LoreStore: LoreRepo {
    async fn stage(&self, paths: &[&str]) -> Result<()>;
    async fn unstage(&self, paths: &[&str]) -> Result<()>;

    /// `cas_revision` is the revision the caller last observed on the branch.
    /// Returns `Err` mapping to HTTP 409 if the branch has moved.
    async fn commit(
        &self,
        message: &str,
        cas_revision: &str,
    ) -> Result<String>;

    async fn create_branch(&self, name: &str, from_revision: &str) -> Result<()>;
    async fn switch_branch(&self, name: &str) -> Result<()>;
    async fn lock(&self, path: &str) -> Result<()>;
    async fn unlock(&self, path: &str) -> Result<()>;
    async fn push(&self, branch: &str) -> Result<()>;
}
