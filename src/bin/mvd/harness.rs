//! Detect the agentic harness driving the current `mvd` invocation.
//!
//! Goal: cheap, best-effort identification of who is calling us
//! (Cursor, Claude Code, Codex, Aider, Continue, Warp, raw shell, …).
//!
//! Detection priority:
//!   1. `MVD_HARNESS` env (explicit override; CLI `--harness` flag overrides this too)
//!   2. Harness-specific env-var fingerprints
//!   3. Parent process name (`ps -o comm= -p <ppid>` on Unix)
//!   4. Fallback: `"shell"`
//!
//! All detection is non-blocking and never errors.

use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct Harness {
    pub name: String,
    pub version: Option<String>,
}

impl Harness {
    pub fn detect() -> Self {
        // 1. Explicit override.
        if let Ok(name) = std::env::var("MVD_HARNESS") {
            let name = name.trim();
            if !name.is_empty() {
                return Self {
                    name: name.to_string(),
                    version: std::env::var("MVD_HARNESS_VERSION").ok(),
                };
            }
        }

        // 2. Env fingerprints (ordered: most specific first).
        if env_present("CLAUDECODE") || env_present("CLAUDE_CODE_ENTRYPOINT") || env_present("CLAUDE_CODE_VERSION") {
            return named("claude-code", env_first(&["CLAUDE_CODE_VERSION"]));
        }
        if env_present("CURSOR_TRACE_ID") || env_present("CURSOR_PROJECT_DIR") || env_present("CURSOR_AGENT") {
            return named("cursor", env_first(&["CURSOR_VERSION"]));
        }
        if env_present("CODEX_HOME") || env_present("CODEX_SANDBOX") || env_present("CODEX_AGENT") {
            return named("codex", env_first(&["CODEX_VERSION"]));
        }
        if env_present("AIDER_CHAT_HISTORY_FILE") || env_present("AIDER_VERSION") {
            return named("aider", env_first(&["AIDER_VERSION"]));
        }
        if env_present("CONTINUE_SESSION_ID") || env_present("CONTINUE_GLOBAL_DIR") {
            return named("continue", env_first(&["CONTINUE_VERSION"]));
        }
        if env_present("WARP_IS_LOCAL_SHELL_SESSION") || env_present("WARP_HONOR_PS1") {
            return named("warp", env_first(&["WARP_VERSION"]));
        }
        if std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
            return named("vscode", env_first(&["TERM_PROGRAM_VERSION"]));
        }

        // 3. Parent process name.
        if let Some(parent) = parent_process_name() {
            let lower = parent.to_lowercase();
            if lower.contains("cursor") {
                return named("cursor", None);
            }
            if lower.contains("claude") {
                return named("claude-code", None);
            }
            if lower.contains("codex") {
                return named("codex", None);
            }
            if lower.contains("aider") {
                return named("aider", None);
            }
            if lower.contains("zed") {
                return named("zed", None);
            }
            if lower.contains("cline") {
                return named("cline", None);
            }
            if lower.contains("roo") {
                return named("roo", None);
            }
            // Known shells → fallback to "shell".
            if matches!(lower.as_str(), "bash" | "zsh" | "fish" | "sh" | "dash" | "ksh") {
                return named("shell", None);
            }
            return named(&format!("other:{lower}"), None);
        }

        // 4. Fallback.
        named("shell", None)
    }
}

fn named(name: &str, version: Option<String>) -> Harness {
    Harness { name: name.to_string(), version }
}

fn env_present(key: &str) -> bool {
    std::env::var_os(key).map(|v| !v.is_empty()).unwrap_or(false)
}

fn env_first(keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty())
}

#[cfg(unix)]
fn parent_process_name() -> Option<String> {
    let self_pid = std::process::id();
    let ppid_out = Command::new("ps")
        .args(["-o", "ppid=", "-p", &self_pid.to_string()])
        .output()
        .ok()?;
    if !ppid_out.status.success() {
        return None;
    }
    let ppid = String::from_utf8(ppid_out.stdout).ok()?.trim().to_string();
    if ppid.is_empty() || ppid == "0" || ppid == "1" {
        return None;
    }

    let comm_out = Command::new("ps")
        .args(["-o", "comm=", "-p", &ppid])
        .output()
        .ok()?;
    if !comm_out.status.success() {
        return None;
    }
    let s = String::from_utf8(comm_out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        return None;
    }
    // Strip path; `ps` may return `-zsh` for login shells.
    let last = s.rsplit('/').next().unwrap_or(&s);
    Some(last.trim_start_matches('-').to_string())
}

#[cfg(not(unix))]
fn parent_process_name() -> Option<String> {
    None
}
