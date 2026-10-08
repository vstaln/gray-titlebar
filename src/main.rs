//! gray-titlebar — the terminal title reflects what the agent is doing.
//!
//! Port of pi's `titlebar-spinner` extension. Pi animated a braille spinner
//! on a timer; a sidecar can't paint between wire lines, so the title is
//! event-driven instead: `pre_tool` shows `⬡ gray — <tool>`, `post_tool`
//! keeps the tool name with a ✓/✗ result mark, and `turn_end` (and
//! shutdown) restores `⬡ gray — ready`. Titles are OSC 2 written to
//! /dev/tty — stdout is the JSON wire, never the terminal.
//!
//! `/titlebar` reports status; `/titlebar on|off` toggles, persisted at
//! ~/.gray/titlebar/disabled. Headless (no tty) → every write is a no-op.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

const READY: &str = "⬡ gray — ready";
const IDLE: &str = "gray";

fn manifest() -> Value {
    json!({
        "name": "titlebar",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "1.1",
        "tools": [],
        "commands": ["/titlebar"],
    })
}

/// Strip ESC and BEL so a hostile or sloppy tool name can't break out of
/// the OSC payload or inject its own sequences.
fn osc_safe(s: &str) -> String {
    s.chars().filter(|c| *c != '\x1b' && *c != '\x07').collect()
}

/// The full OSC 2 sequence for a title.
fn title_seq(title: &str) -> String {
    format!("\x1b]2;{}\x07", osc_safe(title))
}

/// Write to the controlling terminal, bypassing the NDJSON stdout wire.
/// Returns false when no tty is reachable (cron runs, `gray -p` subagents).
fn tty_write(s: &str) -> bool {
    std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .and_then(|mut f| f.write_all(s.as_bytes()).and_then(|_| f.flush()))
        .is_ok()
}

fn set_title(title: &str) -> bool {
    tty_write(&title_seq(title))
}

fn state_dir() -> PathBuf {
    let home = std::env::var_os("GRAY_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".gray")))
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("titlebar")
}

fn enabled_in(dir: &Path) -> bool {
    !dir.join("disabled").exists()
}

fn enabled() -> bool {
    enabled_in(&state_dir())
}

/// Small rolling state for the current turn: last tool seen + call count.
#[derive(Default)]
struct Turn {
    tool: String,
    calls: usize,
}

impl Turn {
    fn pre_tool(&mut self, name: &str) {
        self.calls += 1;
        if !name.is_empty() {
            self.tool = name.to_string();
        }
        let suffix = if self.calls > 1 {
            format!(" ×{}", self.calls)
        } else {
            String::new()
        };
        let title = if self.tool.is_empty() {
            "⬡ gray".to_string()
        } else {
            format!("⬡ gray — {}{suffix}", self.tool)
        };
        set_title(&title);
    }

    fn post_tool(&mut self, name: &str, is_error: bool) {
        if !name.is_empty() {
            self.tool = name.to_string();
        }
        if self.tool.is_empty() {
            return;
        }
        let mark = if is_error { "✗" } else { "✓" };
        set_title(&format!("⬡ gray — {} {mark}", self.tool));
    }
}

/// `/titlebar …` — `argv` excludes the command name. `dir` is the state
/// dir (parameterised so tests never touch the real ~/.gray).
fn run_command(argv: &[&str], dir: &Path) -> String {
    match argv.first().copied() {
        Some("off") => match std::fs::create_dir_all(dir)
            .and_then(|_| std::fs::write(dir.join("disabled"), b""))
        {
            Ok(()) => {
                set_title(IDLE);
                "titlebar off — terminal title left alone".into()
            }
            Err(e) => format!("couldn't disable: {e}"),
        },
        Some("on") => match std::fs::remove_file(dir.join("disabled")).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        }) {
            Ok(()) => {
                set_title(READY);
                "titlebar on".into()
            }
            Err(e) => format!("couldn't enable: {e}"),
        },
        _ => format!(
            "gray-titlebar {} — {} — OSC 2 titles on tool calls, \"{READY}\" when idle. \
             /titlebar on|off",
            env!("CARGO_PKG_VERSION"),
            if enabled_in(dir) { "on" } else { "off" },
        ),
    }
}

/// One request → `Some(reply)`, or `None` for notifications. The bool asks
/// the loop to exit after writing the reply.
fn handle(req: &Value, turn: &mut Turn) -> (Option<Value>, bool) {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let Some(id) = id else {
        if method == "event/notify" && enabled() {
            match params.get("type").and_then(Value::as_str) {
                Some("pre_tool") => {
                    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                    turn.pre_tool(name);
                }
                Some("post_tool") => {
                    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                    let is_error = params
                        .get("is_error")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    turn.post_tool(name, is_error);
                }
                Some("turn_end") => {
                    *turn = Turn::default();
                    set_title(READY);
                }
                _ => {}
            }
        }
        // Notifications also arrive on shutdown — restore a neutral title.
        if method == "plugin/shutdown" && enabled() {
            set_title(IDLE);
        }
        return (None, method == "plugin/shutdown");
    };
    let result = match method {
        "plugin/manifest" => manifest(),
        "command/run" => {
            let argv: Vec<&str> = params
                .get("argv")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            json!({ "text": run_command(&argv, &state_dir()) })
        }
        "plugin/shutdown" => {
            if enabled() {
                set_title(IDLE);
            }
            return (Some(json!({ "id": id, "result": {} })), true);
        }
        _ => {
            let error = json!({ "code": -32601, "message": "method not found" });
            return (Some(json!({ "id": id, "error": error })), false);
        }
    };
    (Some(json!({ "id": id, "result": result })), false)
}

fn main() -> std::io::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("manifest") {
        println!("{}", manifest());
        return Ok(());
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut turn = Turn::default();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(req) = serde_json::from_str::<Value>(&line) else { continue };
        let (reply, exit) = handle(&req, &mut turn);
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
        if exit {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(turn: &mut Turn, method: &str, params: Value) -> Value {
        handle(&json!({ "id": 1, "method": method, "params": params }), turn)
            .0
            .unwrap()
    }

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gray-titlebar-test-{}-{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn manifest_has_no_tools_and_one_command() {
        let mut t = Turn::default();
        let m = call(&mut t, "plugin/manifest", Value::Null)["result"].clone();
        assert_eq!(m["name"], "titlebar");
        assert_eq!(m["protocol"], "1.1");
        assert_eq!(m["commands"], json!(["/titlebar"]));
        assert_eq!(m["tools"], json!([]));
    }

    #[test]
    fn notifications_are_silent_and_dont_exit() {
        let mut t = Turn::default();
        for ty in ["pre_tool", "post_tool", "turn_end"] {
            let (reply, exit) = handle(
                &json!({"method": "event/notify", "params": {"type": ty, "name": "bash", "session": {}}}),
                &mut t,
            );
            assert!(reply.is_none() && !exit);
        }
    }

    #[test]
    fn turn_state_tracks_last_tool_and_resets() {
        let mut t = Turn::default();
        handle(
            &json!({"method":"event/notify","params":{"type":"pre_tool","name":"bash"}}),
            &mut t,
        );
        assert_eq!(t.tool, "bash");
        assert_eq!(t.calls, 1);
        handle(
            &json!({"method":"event/notify","params":{"type":"turn_end"}}),
            &mut t,
        );
        assert_eq!(t.calls, 0);
    }

    #[test]
    fn osc_strips_escape_and_bel() {
        assert_eq!(osc_safe("a\x1b]2;x\x07b"), "a]2;xb");
        assert_eq!(title_seq("e\x1b]0;\x07vil"), "\x1b]2;e]0;vil\x07");
    }

    #[test]
    fn toggle_persists_under_state_dir() {
        let d = tmpdir("toggle");
        assert!(enabled_in(&d));
        assert!(run_command(&["off"], &d).contains("off"));
        assert!(!enabled_in(&d));
        assert!(d.join("disabled").exists());
        assert!(run_command(&["on"], &d).contains("on"));
        assert!(enabled_in(&d));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn bare_command_reports_state() {
        let d = tmpdir("status");
        let s = run_command(&[], &d);
        assert!(s.contains("gray-titlebar") && s.contains("— on"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn shutdown_replies_then_exits() {
        let mut t = Turn::default();
        let (reply, exit) = handle(&json!({ "id": 2, "method": "plugin/shutdown" }), &mut t);
        assert!(reply.is_some() && exit);
    }
}
