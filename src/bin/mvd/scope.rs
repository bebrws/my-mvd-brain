//! Scope flags shared across commands.
//!
//! Two CLI arg groups:
//!   * `ScopeWrite` — used by put/put_many/enrich to choose how each new frame
//!     is stamped (repo / branch / harness).
//!   * `ScopeRead`  — used by find/vec/ask/timeline/recent/stats to choose
//!     which frames are visible.
//!
//! Both default to auto-detection from the working directory's git context and
//! the harness env. Explicit flags always override; `--no-scope` (write) and
//! `--all-repos` (read) are escape hatches.

use clap::Args;

use crate::git_context::GitContext;
use crate::harness::Harness;

/// Load the user's config file (`~/.config/mvd/config.json`) as a flat map.
/// Returns an empty map on any error; config is best-effort.
fn load_config_map() -> serde_json::Map<String, serde_json::Value> {
    let Ok(path) = crate::common::config_file_path() else { return serde_json::Map::new() };
    if !path.exists() { return serde_json::Map::new(); }
    let Ok(data) = std::fs::read_to_string(&path) else { return serde_json::Map::new() };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) else {
        return serde_json::Map::new();
    };
    val.as_object().cloned().unwrap_or_default()
}

/// `scope.default = off` disables auto-detection. Anything else (including
/// missing) leaves the default behavior on.
fn auto_detection_enabled() -> bool {
    let cfg = load_config_map();
    !matches!(
        cfg.get("scope.default").and_then(|v| v.as_str()),
        Some("off")
    )
}

/// Apply `scope.repo_alias.<remote>` rewrites — if the user has configured
/// `mvd config set scope.repo_alias.github.com/old/repo new/name`, every
/// occurrence of `github.com/old/repo` is rewritten to `new/name`.
fn apply_repo_alias(repo: Option<String>) -> Option<String> {
    let raw = repo?;
    let cfg = load_config_map();
    let key = format!("scope.repo_alias.{raw}");
    if let Some(alias) = cfg.get(&key).and_then(|v| v.as_str()) {
        return Some(alias.to_string());
    }
    Some(raw)
}

#[derive(Args, Clone, Debug, Default)]
pub struct ScopeWrite {
    /// Override repository identifier (default: detected from git).
    #[arg(long, value_name = "ID")]
    pub repo: Option<String>,
    /// Override branch (default: detected from git).
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,
    /// Override harness identifier (default: detected from env, e.g. cursor).
    #[arg(long, value_name = "NAME")]
    pub harness: Option<String>,
    /// Suppress auto-stamping of repo/branch/harness on write.
    #[arg(long)]
    pub no_scope: bool,
}

#[derive(Args, Clone, Debug, Default)]
pub struct ScopeRead {
    /// Restrict to this repository (default: current repo).
    #[arg(long, value_name = "ID")]
    pub repo: Option<String>,
    /// Restrict to this branch (default: current branch).
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,
    /// Restrict to this harness.
    #[arg(long, value_name = "NAME")]
    pub harness: Option<String>,
    /// Span all branches (current repo only).
    #[arg(long)]
    pub all_branches: bool,
    /// Span every repository / branch / harness (alias: --global).
    #[arg(long, alias = "global")]
    pub all_repos: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedWriteScope {
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub harness: Option<String>,
    pub harness_version: Option<String>,
}

impl ScopeWrite {
    pub fn resolve(&self) -> ResolvedWriteScope {
        if self.no_scope || !auto_detection_enabled() {
            return ResolvedWriteScope {
                repo: self.repo.clone(),
                branch: self.branch.clone(),
                harness: self.harness.clone(),
                harness_version: None,
            };
        }
        let git = GitContext::detect();
        let h = Harness::detect();
        ResolvedWriteScope {
            repo: apply_repo_alias(self.repo.clone().or(git.repo)),
            branch: self.branch.clone().or(git.branch),
            harness: Some(self.harness.clone().unwrap_or(h.name)),
            harness_version: h.version,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedReadScope {
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub harness: Option<String>,
}

impl ScopeRead {
    /// Resolve filter values. `--all-repos` clears repo + branch entirely
    /// (unless explicit `--repo` / `--branch` were also supplied).
    /// Outside a git repo we behave like `--all-repos` so the CLI still works.
    /// `scope.default = off` in `mvd config` also disables auto-filtering.
    pub fn resolve(&self) -> ResolvedReadScope {
        if self.all_repos {
            return ResolvedReadScope {
                repo: apply_repo_alias(self.repo.clone()),
                branch: self.branch.clone(),
                harness: self.harness.clone(),
            };
        }
        if !auto_detection_enabled() {
            return ResolvedReadScope {
                repo: apply_repo_alias(self.repo.clone()),
                branch: self.branch.clone(),
                harness: self.harness.clone(),
            };
        }
        let git = GitContext::detect();
        let detected_repo = git.repo.clone();
        let resolved_repo = apply_repo_alias(self.repo.clone().or(git.repo));

        // If the user pointed `--repo` at a different repository than the cwd,
        // their current branch is irrelevant — span all branches there unless
        // they explicitly opt back in with `--branch`.
        let repo_matches_cwd = match (&self.repo, &detected_repo) {
            (Some(r), Some(d)) => r == d,
            (None, _) => true,
            _ => false,
        };

        let branch = if self.all_branches {
            None
        } else if let Some(b) = self.branch.clone() {
            Some(b)
        } else if repo_matches_cwd {
            git.branch
        } else {
            None
        };

        ResolvedReadScope {
            repo: resolved_repo,
            branch,
            harness: self.harness.clone(),
        }
    }
}

impl ResolvedReadScope {
    /// Returns `true` if no filter is active (read all frames).
    pub fn is_unfiltered(&self) -> bool {
        self.repo.is_none() && self.branch.is_none() && self.harness.is_none()
    }

    /// Test whether a frame's `extra_metadata` matches this scope.
    pub fn matches(&self, extra_metadata: &std::collections::BTreeMap<String, String>) -> bool {
        if let Some(ref want) = self.repo {
            if extra_metadata.get("repo").map(|v| v.as_str()) != Some(want.as_str()) {
                return false;
            }
        }
        if let Some(ref want) = self.branch {
            if extra_metadata.get("branch").map(|v| v.as_str()) != Some(want.as_str()) {
                return false;
            }
        }
        if let Some(ref want) = self.harness {
            if extra_metadata.get("harness").map(|v| v.as_str()) != Some(want.as_str()) {
                return false;
            }
        }
        true
    }

    /// Drop search hits whose frame metadata doesn't match this scope.
    /// Hits without metadata or with no `extra_metadata` are excluded when a
    /// filter is active — they have no way to match.
    pub fn filter_hits(&self, hits: &mut Vec<memvid_core::SearchHit>) {
        if self.is_unfiltered() {
            return;
        }
        hits.retain(|h| match &h.metadata {
            Some(m) => self.matches(&m.extra_metadata),
            None => false,
        });
    }

    /// Short, user-facing description of the active scope (for non-JSON output).
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref r) = self.repo { parts.push(format!("repo={r}")); }
        if let Some(ref b) = self.branch { parts.push(format!("branch={b}")); }
        if let Some(ref h) = self.harness { parts.push(format!("harness={h}")); }
        if parts.is_empty() { "all repos".to_string() } else { parts.join(" ") }
    }
}

/// Apply resolved write-scope to a built `PutOptions`.
///
/// Each value lands in **both** `extra_metadata` (structured, exact-match
/// filters) and `tags` as `key:value` strings (indexed by Tantivy and trivially
/// matchable). We deliberately do not use `PutOptionsBuilder::tag()` — that
/// helper only stores the key in `tags`, which would collide with our other
/// metadata.
pub fn apply_write_scope(
    options: &mut memvid_core::PutOptions,
    scope: &ResolvedWriteScope,
) {
    if let Some(ref repo) = scope.repo {
        options.extra_metadata.insert("repo".into(), repo.clone());
        options.tags.push(format!("repo:{repo}"));
    }
    if let Some(ref branch) = scope.branch {
        options.extra_metadata.insert("branch".into(), branch.clone());
        options.tags.push(format!("branch:{branch}"));
    }
    if let Some(ref harness) = scope.harness {
        options.extra_metadata.insert("harness".into(), harness.clone());
        options.tags.push(format!("harness:{harness}"));
    }
    if let Some(ref version) = scope.harness_version {
        options.extra_metadata.insert("harness_version".into(), version.clone());
    }
}
