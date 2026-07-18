//! The repo-local `.veilgremlin/` state directory: the one consistent place the hooks and
//! the `vg` CLI keep the vault DB, audit log, layered policy packs, and persisted masked
//! packs (a demask needs the pack a prior mask produced).
//!
//! Layout under the state root (`.veilgremlin/` by default, git-ignored):
//!
//! ```text
//! .veilgremlin/
//!   vault.db                     SQLCipher mapping store (vg-vault)
//!   audit.jsonl                  append-only audit log (vg-audit)
//!   claude-hooks.json            hook config `vg run` writes for Claude Code
//!   policy/
//!     global.policy.json         required layer (written from a default on first use)
//!     repo.policy.json           optional overlay
//!     session.policy.json        optional overlay
//!   packs/
//!     <uuid>.json                one persisted MaskedPack per masked artefact
//! ```
//!
//! Resolution precedence for the state root: an explicit path (the CLI's `--state-dir`),
//! then the `VG_STATE_DIR` environment variable, then the nearest existing `.veilgremlin`
//! walking up from the current directory, else `<cwd>/.veilgremlin`.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

/// Environment variable that pins the state directory (an absolute or cwd-relative path to
/// the `.veilgremlin` dir itself), overriding upward discovery. The CLI's `--state-dir`
/// flag takes precedence over it.
pub const STATE_DIR_ENV: &str = "VG_STATE_DIR";

/// The conventional directory name for the repo-local state dir.
pub const STATE_DIR_NAME: &str = ".veilgremlin";

/// Resolved paths under one state directory.
#[derive(Debug, Clone)]
pub struct StatePaths {
    /// The `.veilgremlin` directory itself.
    root: PathBuf,
    /// The directory the state dir lives in — used to key the repo `Namespace` so
    /// placeholders are stable across invocations in the same working tree.
    repo_root: PathBuf,
}

impl StatePaths {
    /// Resolves the state paths, applying the precedence documented on the module. Does not
    /// create anything on disk — call [`StatePaths::ensure`] for that.
    pub fn resolve(explicit: Option<PathBuf>) -> io::Result<Self> {
        if let Some(dir) = explicit {
            return Ok(Self::rooted_at(dir));
        }
        if let Some(dir) = env::var_os(STATE_DIR_ENV) {
            return Ok(Self::rooted_at(PathBuf::from(dir)));
        }
        let cwd = env::current_dir()?;
        if let Some(found) = discover_upward(&cwd) {
            return Ok(Self::rooted_at(found));
        }
        Ok(Self::rooted_at(cwd.join(STATE_DIR_NAME)))
    }

    /// Builds paths rooted at a specific `.veilgremlin` directory (its parent becomes the
    /// repo root). Absolutised best-effort so a persisted pack's repo namespace does not
    /// depend on the cwd a later `vg demask` runs from.
    pub fn rooted_at(root: PathBuf) -> Self {
        let root = absolutise(root);
        let repo_root = root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.clone());
        Self { root, repo_root }
    }

    /// Creates the state directory and its `policy/` and `packs/` subdirectories if absent.
    pub fn ensure(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.policy_dir())?;
        std::fs::create_dir_all(self.packs_dir())?;
        // Self-gitignore the whole state dir (doubt-pass mitigation): packs hold the full
        // masked text of prompts/tool IO in plaintext, and an accidentally committed
        // `.veilgremlin/` would publish packs + audit log + vault file. A `*` ignore
        // inside the dir needs no repo `.gitignore` edit. Written only if absent, so a
        // deliberate operator override survives.
        let gitignore = self.root.join(".gitignore");
        if !gitignore.exists() {
            std::fs::write(&gitignore, "*\n")?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn vault_db(&self) -> PathBuf {
        self.root.join("vault.db")
    }

    pub fn audit_log(&self) -> PathBuf {
        self.root.join("audit.jsonl")
    }

    pub fn hook_config(&self) -> PathBuf {
        self.root.join("claude-hooks.json")
    }

    pub fn policy_dir(&self) -> PathBuf {
        self.root.join("policy")
    }

    pub fn global_policy(&self) -> PathBuf {
        self.policy_dir().join("global.policy.json")
    }

    pub fn repo_policy(&self) -> PathBuf {
        self.policy_dir().join("repo.policy.json")
    }

    pub fn session_policy(&self) -> PathBuf {
        self.policy_dir().join("session.policy.json")
    }

    pub fn packs_dir(&self) -> PathBuf {
        self.root.join("packs")
    }
}

/// Walks up from `start` looking for an existing `.veilgremlin` directory, returning it if
/// found (so a hook fired deep in a subdirectory shares the repo-root state).
fn discover_upward(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join(STATE_DIR_NAME);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Makes `path` absolute against the cwd when it is relative, without touching the
/// filesystem (`std::fs::canonicalize` would require the path to exist and resolve
/// symlinks; we only want a stable, cwd-independent key). Falls back to the input on any
/// error.
fn absolutise(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    match env::current_dir() {
        Ok(cwd) => cwd.join(path),
        Err(_) => path,
    }
}
