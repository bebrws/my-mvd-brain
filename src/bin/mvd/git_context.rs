//! Detect the current git repository and branch from `cwd` (or a provided path).
//!
//! Used to auto-stamp frames written via `mvd put` so memories know which
//! project / branch they belong to. All functions are best-effort: any failure
//! (no git, no remote, detached HEAD, command missing) returns `None` rather
//! than erroring — memory operations must never block on git.

use std::path::Path;
use std::process::Command;

/// Detected git context for the current working directory.
#[derive(Debug, Clone, Default)]
pub struct GitContext {
    /// Stable repository identifier — host/owner/name when a remote exists,
    /// else `local/<toplevel-basename>` as a fallback.
    pub repo: Option<String>,
    /// Current branch (`None` on detached HEAD or non-git).
    pub branch: Option<String>,
    /// Absolute path to the git toplevel, if inside a repo.
    pub toplevel: Option<String>,
}

impl GitContext {
    /// Detect from the process's current working directory.
    pub fn detect() -> Self {
        Self::detect_at(std::env::current_dir().ok().as_deref())
    }

    /// Detect from a specific path. `None` means "use cwd".
    pub fn detect_at(path: Option<&Path>) -> Self {
        let toplevel = run_git(path, &["rev-parse", "--show-toplevel"]);
        let branch = run_git(path, &["rev-parse", "--abbrev-ref", "HEAD"])
            .filter(|b| b != "HEAD"); // detached HEAD
        let remote = run_git(path, &["remote", "get-url", "origin"]);

        let repo = remote
            .as_deref()
            .and_then(normalize_remote)
            .or_else(|| {
                toplevel.as_deref().and_then(|t| {
                    Path::new(t)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| format!("local/{n}"))
                })
            });

        Self { repo, branch, toplevel }
    }

    /// `true` if we detected a git repo (toplevel resolved).
    #[allow(dead_code)]
    pub fn is_git(&self) -> bool {
        self.toplevel.is_some()
    }
}

fn run_git(cwd: Option<&Path>, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Turn a remote URL into a stable `host/owner/name` identifier.
///
/// Supports the common forms:
///   - `git@github.com:owner/name.git`         → `github.com/owner/name`
///   - `https://github.com/owner/name.git`     → `github.com/owner/name`
///   - `https://user@gitlab.com/owner/name`    → `gitlab.com/owner/name`
///   - `ssh://git@host:22/owner/name.git`      → `host/owner/name`
fn normalize_remote(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // Strip scheme.
    let after_scheme = match raw.split_once("://") {
        Some((_scheme, rest)) => rest.to_string(),
        None => {
            // SCP-style: git@host:owner/name.git
            let (head, tail) = raw.split_once(':')?;
            let host = head.rsplit_once('@').map(|(_, h)| h).unwrap_or(head);
            return finalize_host_path(host, tail);
        }
    };

    // Drop optional `user@`
    let host_and_rest = after_scheme
        .split_once('@')
        .map(|(_, r)| r.to_string())
        .unwrap_or(after_scheme);

    let (host_with_port, path) = host_and_rest.split_once('/')?;
    // Drop optional `:port` from host.
    let host = host_with_port
        .split_once(':')
        .map(|(h, _)| h)
        .unwrap_or(host_with_port);
    finalize_host_path(host, path)
}

fn finalize_host_path(host: &str, path: &str) -> Option<String> {
    let host = host.trim().to_lowercase();
    if host.is_empty() {
        return None;
    }
    let path = path.trim().trim_start_matches('/').trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    if path.is_empty() {
        return None;
    }
    Some(format!("{host}/{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_https() {
        assert_eq!(
            normalize_remote("https://github.com/foo/bar.git").as_deref(),
            Some("github.com/foo/bar")
        );
    }

    #[test]
    fn normalize_scp() {
        assert_eq!(
            normalize_remote("git@github.com:foo/bar.git").as_deref(),
            Some("github.com/foo/bar")
        );
    }

    #[test]
    fn normalize_no_dot_git() {
        assert_eq!(
            normalize_remote("https://gitlab.com/foo/bar").as_deref(),
            Some("gitlab.com/foo/bar")
        );
    }

    #[test]
    fn normalize_user_in_url() {
        assert_eq!(
            normalize_remote("https://user@gitlab.com/foo/bar.git").as_deref(),
            Some("gitlab.com/foo/bar")
        );
    }

    #[test]
    fn normalize_ssh_with_port() {
        assert_eq!(
            normalize_remote("ssh://git@host.example:22/foo/bar.git").as_deref(),
            Some("host.example/foo/bar")
        );
    }

    #[test]
    fn normalize_garbage() {
        assert_eq!(normalize_remote(""), None);
        assert_eq!(normalize_remote("not-a-url"), None);
    }
}
