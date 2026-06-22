use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use toolbelt_types::{
    AssetKind, BranchInfo, EntryKind, FileInfo, FileState, RevisionInfo,
    StatusEntry, TreeEntry,
};

use crate::traits::{LoreRepo, LoreStore};

// ── In-memory fake ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FakeFile {
    content: Vec<u8>,
    hash: String,
}

#[derive(Clone)]
struct FakeRevision {
    id: String,
    parent_ids: Vec<String>,
    author: String,
    message: String,
    timestamp: i64,
    tree: HashMap<String, FakeFile>,
}

struct FakeState {
    name: String,
    default_branch: String,
    branches: HashMap<String, String>, // branch → revision id
    revisions: HashMap<String, FakeRevision>,
    staged: Vec<String>,
    working_tree: HashMap<String, FakeFile>,
    locks: HashMap<String, String>, // path → owner
}

#[derive(Clone)]
pub struct FakeLoreRepo {
    state: Arc<Mutex<FakeState>>,
}

impl FakeLoreRepo {
    pub fn new(repo_name: impl Into<String>) -> Self {
        let root_rev = FakeRevision {
            id: "rev-0000".into(),
            parent_ids: vec![],
            author: "fixture".into(),
            message: "initial".into(),
            timestamp: 0,
            tree: HashMap::new(),
        };

        let mut revisions = HashMap::new();
        revisions.insert(root_rev.id.clone(), root_rev.clone());

        let mut branches = HashMap::new();
        branches.insert("main".into(), root_rev.id.clone());

        Self {
            state: Arc::new(Mutex::new(FakeState {
                name: repo_name.into(),
                default_branch: "main".into(),
                branches,
                revisions,
                staged: vec![],
                working_tree: HashMap::new(),
                locks: HashMap::new(),
            })),
        }
    }

    /// Seed the fake with a file so tests have something to browse.
    pub fn seed_file(
        &self,
        path: impl Into<String>,
        content: impl Into<Vec<u8>>,
    ) -> &Self {
        let path = path.into();
        let content = content.into();
        let hash = format!("sha256:{:x}", content.len()); // not a real hash — fine for tests
        let file = FakeFile { content, hash };

        let mut s = self.state.lock().unwrap();
        s.working_tree.insert(path.clone(), file.clone());

        // also put it in rev-0000 so tree browsing works
        if let Some(rev) = s.revisions.get_mut("rev-0000") {
            rev.tree.insert(path, file);
        }

        self
    }
}

fn asset_kind_from_path(path: &str) -> AssetKind {
    match path.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => AssetKind::Markdown,
        "rs" | "ts" | "tsx" | "js" | "toml" | "yaml" | "yml" | "json"
        | "txt" | "gitignore" => AssetKind::Text,
        "png" | "jpg" | "jpeg" | "tga" | "tiff" | "exr" | "psd" => {
            AssetKind::Image
        }
        "glb" | "gltf" | "fbx" | "obj" => AssetKind::Model3d,
        "blend" => AssetKind::Blend,
        "uasset" | "umap" => AssetKind::Uasset,
        _ => AssetKind::Unknown,
    }
}

#[async_trait::async_trait]
impl LoreRepo for FakeLoreRepo {
    async fn repo_name(&self) -> Result<String> {
        Ok(self.state.lock().unwrap().name.clone())
    }

    async fn default_branch(&self) -> Result<String> {
        Ok(self.state.lock().unwrap().default_branch.clone())
    }

    async fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let s = self.state.lock().unwrap();
        let default = s.default_branch.clone();
        Ok(s.branches
            .iter()
            .map(|(name, head)| BranchInfo {
                name: name.clone(),
                head_revision: head.clone(),
                is_default: name == &default,
            })
            .collect())
    }

    async fn list_tree(&self, revision: &str, path: &str) -> Result<Vec<TreeEntry>> {
        let s = self.state.lock().unwrap();
        let rev = s
            .revisions
            .get(revision)
            .ok_or_else(|| anyhow::anyhow!("revision not found: {revision}"))?;

        let prefix = if path.is_empty() || path == "/" {
            String::new()
        } else {
            format!("{}/", path.trim_matches('/'))
        };

        let mut seen_dirs: std::collections::HashSet<String> = Default::default();
        let mut entries = vec![];

        for (file_path, file) in &rev.tree {
            if !file_path.starts_with(&prefix) {
                continue;
            }
            let rest = &file_path[prefix.len()..];
            if rest.is_empty() {
                continue;
            }

            if let Some(slash) = rest.find('/') {
                let dir_name = rest[..slash].to_string();
                if seen_dirs.insert(dir_name.clone()) {
                    entries.push(TreeEntry {
                        name: dir_name.clone(),
                        path: format!("{}{}", prefix, dir_name),
                        kind: EntryKind::Directory,
                        size: None,
                        content_hash: None,
                    });
                }
            } else {
                entries.push(TreeEntry {
                    name: rest.to_string(),
                    path: file_path.clone(),
                    kind: EntryKind::File,
                    size: Some(file.content.len() as u64),
                    content_hash: Some(file.hash.clone()),
                });
            }
        }

        Ok(entries)
    }

    async fn file_info(&self, revision: &str, path: &str) -> Result<FileInfo> {
        let s = self.state.lock().unwrap();
        let rev = s
            .revisions
            .get(revision)
            .ok_or_else(|| anyhow::anyhow!("revision not found: {revision}"))?;
        let file = rev
            .tree
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("file not found: {path}"))?;
        Ok(FileInfo {
            path: path.to_string(),
            size: file.content.len() as u64,
            content_hash: file.hash.clone(),
            asset_kind: asset_kind_from_path(path),
            lock_owner: s.locks.get(path).cloned(),
        })
    }

    async fn file_content(
        &self,
        revision: &str,
        path: &str,
        range: Option<(u64, u64)>,
    ) -> Result<Vec<u8>> {
        let s = self.state.lock().unwrap();
        let rev = s
            .revisions
            .get(revision)
            .ok_or_else(|| anyhow::anyhow!("revision not found: {revision}"))?;
        let file = rev
            .tree
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("file not found: {path}"))?;

        if let Some((start, end)) = range {
            Ok(file.content[start as usize..end as usize].to_vec())
        } else {
            Ok(file.content.clone())
        }
    }

    async fn list_revisions(
        &self,
        branch: &str,
        limit: usize,
        _cursor: Option<&str>,
    ) -> Result<Vec<RevisionInfo>> {
        let s = self.state.lock().unwrap();
        let head = s
            .branches
            .get(branch)
            .ok_or_else(|| anyhow::anyhow!("branch not found: {branch}"))?
            .clone();

        let mut result = vec![];
        let mut current = Some(head);
        while let Some(id) = current {
            if result.len() >= limit {
                break;
            }
            if let Some(rev) = s.revisions.get(&id) {
                current = rev.parent_ids.first().cloned();
                result.push(RevisionInfo {
                    id: rev.id.clone(),
                    parent_ids: rev.parent_ids.clone(),
                    author: rev.author.clone(),
                    message: rev.message.clone(),
                    timestamp: rev.timestamp,
                });
            } else {
                break;
            }
        }
        Ok(result)
    }

    async fn revision_parents(&self, revision: &str) -> Result<Vec<String>> {
        let s = self.state.lock().unwrap();
        Ok(s.revisions
            .get(revision)
            .map(|r| r.parent_ids.clone())
            .unwrap_or_default())
    }

    async fn status(&self) -> Result<Vec<StatusEntry>> {
        let s = self.state.lock().unwrap();
        Ok(s.staged
            .iter()
            .map(|p| StatusEntry {
                path: p.clone(),
                state: FileState::Modified,
            })
            .collect())
    }

    async fn diff_paths(
        &self,
        from_revision: &str,
        to_revision: &str,
    ) -> Result<Vec<(String, FileState)>> {
        let s = self.state.lock().unwrap();
        let from = s
            .revisions
            .get(from_revision)
            .ok_or_else(|| anyhow::anyhow!("revision not found: {from_revision}"))?;
        let to = s
            .revisions
            .get(to_revision)
            .ok_or_else(|| anyhow::anyhow!("revision not found: {to_revision}"))?;

        let mut diffs = vec![];
        for path in to.tree.keys() {
            if !from.tree.contains_key(path) {
                diffs.push((path.clone(), FileState::Added));
            } else if from.tree[path].hash != to.tree[path].hash {
                diffs.push((path.clone(), FileState::Modified));
            }
        }
        for path in from.tree.keys() {
            if !to.tree.contains_key(path) {
                diffs.push((path.clone(), FileState::Deleted));
            }
        }
        Ok(diffs)
    }
}

#[async_trait::async_trait]
impl LoreStore for FakeLoreRepo {
    async fn stage(&self, paths: &[&str]) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        for p in paths {
            if !s.staged.iter().any(|x| x == p) {
                s.staged.push(p.to_string());
            }
        }
        Ok(())
    }

    async fn unstage(&self, paths: &[&str]) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.staged.retain(|p| !paths.contains(&p.as_str()));
        Ok(())
    }

    async fn commit(&self, message: &str, cas_revision: &str) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let branch_head = s
            .branches
            .get(&s.default_branch.clone())
            .cloned()
            .unwrap_or_default();

        if branch_head != cas_revision {
            bail!("branch moved");
        }

        let new_id = format!("rev-{:04x}", s.revisions.len());
        let parent_rev = s.revisions.get(&branch_head).cloned();
        let tree = parent_rev
            .map(|r| {
                let mut t = r.tree.clone();
                for path in &s.staged {
                    if let Some(f) = s.working_tree.get(path) {
                        t.insert(path.clone(), f.clone());
                    }
                }
                t
            })
            .unwrap_or_default();

        let rev = FakeRevision {
            id: new_id.clone(),
            parent_ids: vec![branch_head.clone()],
            author: "fixture".into(),
            message: message.to_string(),
            timestamp: 0,
            tree,
        };

        s.revisions.insert(new_id.clone(), rev);
        let default = s.default_branch.clone();
        s.branches.insert(default, new_id.clone());
        s.staged.clear();

        Ok(new_id)
    }

    async fn create_branch(&self, name: &str, from_revision: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.branches.insert(name.to_string(), from_revision.to_string());
        Ok(())
    }

    async fn switch_branch(&self, name: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        if !s.branches.contains_key(name) {
            bail!("branch not found: {name}");
        }
        s.default_branch = name.to_string();
        Ok(())
    }

    async fn lock(&self, path: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.locks.insert(path.to_string(), "fixture".into());
        Ok(())
    }

    async fn unlock(&self, path: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.locks.remove(path);
        Ok(())
    }

    async fn push(&self, _branch: &str) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_round_trip() {
        let repo = FakeLoreRepo::new("test-repo");
        repo.seed_file("README.md", b"# hello".to_vec());

        assert_eq!(repo.repo_name().await.unwrap(), "test-repo");

        let tree = repo.list_tree("rev-0000", "").await.unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "README.md");

        let content = repo
            .file_content("rev-0000", "README.md", None)
            .await
            .unwrap();
        assert_eq!(content, b"# hello");
    }

    #[tokio::test]
    async fn fake_commit_cas() {
        let repo = FakeLoreRepo::new("test-repo");
        repo.seed_file("a.txt", b"v1".to_vec());

        repo.stage(&["a.txt"]).await.unwrap();
        let new_rev = repo.commit("first commit", "rev-0000").await.unwrap();
        assert!(!new_rev.is_empty());

        // CAS failure: branch has moved
        repo.stage(&["a.txt"]).await.unwrap();
        let err = repo.commit("second", "rev-0000").await;
        assert!(err.is_err());
    }
}
