use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn rubric_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rubric-cli"))
}

// ── LSP framing ───────────────────────────────────────────────────────────────

fn encode_lsp(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

fn send(stdin: &mut ChildStdin, body: &str) {
    stdin.write_all(&encode_lsp(body)).unwrap();
    stdin.flush().unwrap();
}

/// Read one LSP message. Returns None on timeout or EOF.
fn recv_one(reader: &mut BufReader<ChildStdout>, timeout: Duration) -> Option<String> {
    use std::sync::mpsc;

    // We can't set a read timeout on BufReader directly, so we check with a simple
    // polling approach using try_read on the raw bytes.
    // Strategy: read headers line-by-line with short blocking reads.
    let deadline = Instant::now() + timeout;

    let mut content_length: Option<usize> = None;

    // Read headers
    loop {
        if Instant::now() > deadline {
            return None;
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return None, // EOF
            Ok(_) => {}
            Err(_) => return None,
        }
        let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break; // blank line = end of headers
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
            content_length = rest.trim().parse().ok();
        }
    }

    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    String::from_utf8(body).ok()
}

/// Drain messages until one matches `method`, or until `timeout` expires.
/// Messages that don't match are silently discarded.
fn recv_until_method(
    reader: &mut BufReader<ChildStdout>,
    method: &str,
    timeout: Duration,
) -> Option<serde_json::Value> {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = match deadline.checked_duration_since(Instant::now()) {
            Some(r) if !r.is_zero() => r,
            _ => return None,
        };
        let msg = recv_one(reader, remaining.min(Duration::from_secs(2)))?;
        let v: serde_json::Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v.get("method").and_then(|m| m.as_str()) == Some(method) {
            return Some(v);
        }
        // Not our method — discard and keep draining
    }
}

// ── Session ───────────────────────────────────────────────────────────────────

struct LspSession {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    req_id: u32,
}

impl LspSession {
    fn start(root_path: &std::path::Path) -> Self {
        let mut child = Command::new(rubric_bin())
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn rubric lsp");

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let root_uri = format!("file://{}", root_path.display());

        // initialize request
        send(&mut stdin, &format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"processId":null,"rootUri":"{root_uri}","capabilities":{{"workspace":{{"didChangeWatchedFiles":{{"dynamicRegistration":false}}}}}}}}}}"#
        ));

        // Read initialize result (has "result" key, not "method")
        // Drain until we find a message with no "method" (the response)
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if Instant::now() > deadline { break; }
            let Some(msg) = recv_one(&mut reader, Duration::from_secs(2)) else { break };
            let v: serde_json::Value = match serde_json::from_str(&msg) { Ok(v) => v, _ => break };
            if v.get("result").is_some() || v.get("error").is_some() {
                break; // got the initialize response
            }
            // It's a notification — keep draining
        }

        // initialized notification (server won't respond to this)
        send(&mut stdin, r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);

        // Brief pause for server to process initialized and start its watcher goroutine
        std::thread::sleep(Duration::from_millis(100));

        LspSession { child, stdin, reader, req_id: 1 }
    }

    fn open(&mut self, uri: &str, text: &str) {
        send(&mut self.stdin, &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{uri}","languageId":"ruby","version":1,"text":{}}}}}}}"#,
            serde_json::to_string(text).unwrap()
        ));
    }

    fn change(&mut self, uri: &str, text: &str) {
        self.req_id += 1;
        send(&mut self.stdin, &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{uri}","version":{}}},"contentChanges":[{{"text":{}}}]}}}}"#,
            self.req_id,
            serde_json::to_string(text).unwrap()
        ));
    }

    fn wait_diagnostics(&mut self, timeout: Duration) -> Option<serde_json::Value> {
        recv_until_method(&mut self.reader, "textDocument/publishDiagnostics", timeout)
    }
}

impl Drop for LspSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_lsp_publishes_diagnostics_on_open() {
    let dir = TempDir::new().unwrap();
    let rb_path = dir.path().join("test.rb");
    std::fs::write(&rb_path, "x = 1   \ny = 2\n").unwrap(); // trailing whitespace

    let mut session = LspSession::start(dir.path());
    let uri = format!("file://{}", rb_path.display());
    session.open(&uri, "x = 1   \ny = 2\n");

    let msg = session
        .wait_diagnostics(Duration::from_secs(5))
        .expect("expected publishDiagnostics");

    let diags = msg["params"]["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty(), "expected at least 1 diagnostic");

    let has_trailing = diags.iter().any(|d| {
        d["message"].as_str().unwrap_or("").contains("TrailingWhitespace")
    });
    assert!(has_trailing, "expected TrailingWhitespace in diagnostics: {:?}", diags);
}

#[test]
fn test_lsp_clears_on_fix() {
    let dir = TempDir::new().unwrap();
    let rb_path = dir.path().join("fix_me.rb");
    std::fs::write(&rb_path, "x = 1   \n").unwrap();

    let mut session = LspSession::start(dir.path());
    let uri = format!("file://{}", rb_path.display());

    session.open(&uri, "x = 1   \n");
    let first = session.wait_diagnostics(Duration::from_secs(5)).expect("first publishDiagnostics");
    assert!(!first["params"]["diagnostics"].as_array().unwrap().is_empty());

    session.change(&uri, "x = 1\n"); // fix the whitespace
    let second = session.wait_diagnostics(Duration::from_secs(5)).expect("second publishDiagnostics");
    let diags = second["params"]["diagnostics"].as_array().unwrap();

    // Trailing whitespace should be gone; other cops may or may not fire on a clean file
    let has_trailing = diags.iter().any(|d| {
        d["message"].as_str().unwrap_or("").contains("TrailingWhitespace")
    });
    assert!(!has_trailing, "TrailingWhitespace should be gone after fix; got: {:?}", diags);
}

#[test]
fn test_lsp_respects_rubric_toml() {
    let dir = TempDir::new().unwrap();
    // Disable TrailingWhitespace entirely
    std::fs::write(
        dir.path().join("rubric.toml"),
        "[rules.\"Layout/TrailingWhitespace\"]\nenabled = false\n",
    ).unwrap();

    let rb_path = dir.path().join("a.rb");
    std::fs::write(&rb_path, "x = 1   \n").unwrap();

    let mut session = LspSession::start(dir.path());
    let uri = format!("file://{}", rb_path.display());
    session.open(&uri, "x = 1   \n");

    let msg = session
        .wait_diagnostics(Duration::from_secs(5))
        .expect("expected publishDiagnostics");
    let diags = msg["params"]["diagnostics"].as_array().unwrap();

    let has_trailing = diags.iter().any(|d| {
        d["message"].as_str().unwrap_or("").contains("TrailingWhitespace")
    });
    assert!(!has_trailing, "TrailingWhitespace should be suppressed by rubric.toml");
}

#[test]
fn test_lsp_respects_todo_suppression() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join(".rubric_todo.toml"),
        "[todo.\"Layout/TrailingWhitespace\"]\nfiles = [\"suppressed.rb\"]\n",
    ).unwrap();

    let rb_path = dir.path().join("suppressed.rb");
    std::fs::write(&rb_path, "x = 1   \n").unwrap();

    let mut session = LspSession::start(dir.path());
    let uri = format!("file://{}", rb_path.display());
    session.open(&uri, "x = 1   \n");

    let msg = session
        .wait_diagnostics(Duration::from_secs(5))
        .expect("expected publishDiagnostics");
    let diags = msg["params"]["diagnostics"].as_array().unwrap();

    let has_trailing = diags.iter().any(|d| {
        d["message"].as_str().unwrap_or("").contains("TrailingWhitespace")
    });
    assert!(!has_trailing, "TrailingWhitespace should be suppressed by .rubric_todo.toml");
}

#[test]
fn test_lsp_latency() {
    let dir = TempDir::new().unwrap();
    // 200-line Ruby file with no violations (just variable assignments)
    let content: String = (0..200)
        .map(|i| format!("x_{i} = {i}\n"))
        .collect();
    let rb_path = dir.path().join("large.rb");
    std::fs::write(&rb_path, &content).unwrap();

    let mut session = LspSession::start(dir.path());
    let uri = format!("file://{}", rb_path.display());

    let t0 = Instant::now();
    session.open(&uri, &content);
    let _ = session.wait_diagnostics(Duration::from_secs(3));
    let elapsed = t0.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "Expected diagnostics within 500ms, took {}ms",
        elapsed.as_millis()
    );
}
