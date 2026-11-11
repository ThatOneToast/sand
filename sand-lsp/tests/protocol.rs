//! Drive the language server over an in-memory LSP connection, exactly as an
//! editor would: initialize, open unsaved buffers, query, edit, and shut down.

use lsp_server::{Connection, Message, Notification, Request, RequestId};
use serde_json::{Value, json};
use std::time::Duration;

const ROOT: &str = "/sand-lsp-test/pack.sand";
const MODULE: &str = "/sand-lsp-test/rewards.sand";

const ROOT_TEXT: &str = r#"#![pack(namespace = "arena", minecraft = "26.2")]
mod rewards;
use rewards::grant;
player state Hero { energy: number = 10 }
#[tick]
fn tick() {
    for player in players() {
        grant();
    }
}
"#;

const MODULE_TEXT: &str = "use crate::Hero;\npub fn grant() {\n    Hero.energy += 5;\n}\n";

struct Client {
    connection: Connection,
    next_id: i32,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Client {
    fn start() -> Self {
        let (server, connection) = Connection::memory();
        let thread = std::thread::spawn(move || sand_lsp::run(server).expect("server runs"));
        let mut client = Client {
            connection,
            next_id: 0,
            thread: Some(thread),
        };
        let init = client.request("initialize", json!({"capabilities": {}}));
        assert_eq!(init["capabilities"]["hoverProvider"], true);
        assert_eq!(
            init["capabilities"]["renameProvider"]["prepareProvider"],
            true
        );
        client.notify("initialized", json!({}));
        client
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = RequestId::from(self.next_id);
        self.connection
            .sender
            .send(Message::Request(Request::new(
                id.clone(),
                method.into(),
                params,
            )))
            .unwrap();
        loop {
            match self.receive() {
                Message::Response(response) if response.id == id => {
                    return match response.response_result {
                        Ok(value) => value,
                        Err(error) => panic!("{method} failed: {}", error.message),
                    };
                }
                _ => continue,
            }
        }
    }

    fn notify(&self, method: &str, params: Value) {
        self.connection
            .sender
            .send(Message::Notification(Notification::new(
                method.into(),
                params,
            )))
            .unwrap();
    }

    fn receive(&self) -> Message {
        self.connection
            .receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("server responds")
    }

    /// Collect diagnostics notifications until one per expected URI arrives.
    fn diagnostics(&self, count: usize) -> Vec<(String, Vec<Value>)> {
        let mut out = Vec::new();
        while out.len() < count {
            if let Message::Notification(n) = self.receive()
                && n.method == "textDocument/publishDiagnostics"
            {
                let uri = n.params["uri"].as_str().unwrap().to_string();
                let items = n.params["diagnostics"].as_array().unwrap().clone();
                out.push((uri, items));
            }
        }
        out
    }

    fn open(&self, path: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri(path), "languageId": "sand", "version": 1, "text": text}}),
        );
    }

    fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.thread.take().unwrap().join().unwrap();
    }
}

fn uri(path: &str) -> String {
    format!("file://{path}")
}

fn position(text: &str, needle: &str) -> Value {
    let offset = text.find(needle).unwrap();
    let line = text[..offset].matches('\n').count();
    let col = offset - text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    json!({"line": line, "character": col})
}

#[test]
fn editor_session_round_trip() {
    let mut client = Client::start();
    client.open(MODULE, MODULE_TEXT);
    // The module alone cannot see the unsaved root yet, so it reports the
    // missing root; opening the root re-analyses the whole package.
    let _ = client.diagnostics(1);
    client.open(ROOT, ROOT_TEXT);
    let published = client.diagnostics(2);
    for (uri, diagnostics) in &published {
        assert!(diagnostics.is_empty(), "{uri}: {diagnostics:#?}");
    }

    let hover = client.request(
        "textDocument/hover",
        json!({"textDocument": {"uri": uri(ROOT)}, "position": position(ROOT_TEXT, "grant();")}),
    );
    let markdown = hover["contents"]["value"].as_str().unwrap();
    assert!(markdown.contains("pub fn grant()"), "{markdown}");
    assert!(markdown.contains("Runs as a **player**"), "{markdown}");

    let definition = client.request(
        "textDocument/definition",
        json!({"textDocument": {"uri": uri(ROOT)}, "position": position(ROOT_TEXT, "grant();")}),
    );
    assert_eq!(definition["uri"], uri(MODULE));
    assert_eq!(
        definition["range"]["start"],
        json!({"line": 1, "character": 7})
    );

    let completion = client.request(
        "textDocument/completion",
        json!({"textDocument": {"uri": uri(MODULE)}, "position": position(MODULE_TEXT, "energy +=")}),
    );
    let labels: Vec<_> = completion
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, vec!["energy"]);

    let rename = client.request(
        "textDocument/rename",
        json!({"textDocument": {"uri": uri(MODULE)}, "position": position(MODULE_TEXT, "Hero.energy"), "newName": "Champion"}),
    );
    let changes = rename["changes"].as_object().unwrap();
    assert_eq!(changes[&uri(ROOT)].as_array().unwrap().len(), 1);
    assert_eq!(changes[&uri(MODULE)].as_array().unwrap().len(), 2);

    let symbols = client.request(
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": uri(ROOT)}}),
    );
    let names: Vec<_> = symbols
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["rewards", "Hero", "tick"]);

    let hints = client.request(
        "textDocument/inlayHint",
        json!({"textDocument": {"uri": uri(MODULE)}, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 0}}}),
    );
    assert_eq!(hints[0]["label"], ": player");

    // An incremental edit that calls player work straight from #[tick].
    let line = 7;
    client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri(ROOT), "version": 2},
            "contentChanges": [{
                "range": {"start": {"line": line, "character": 0}, "end": {"line": line + 2, "character": 0}},
                "text": "    }\n    grant();\n"
            }]
        }),
    );
    let published = client.diagnostics(2);
    let root = published.iter().find(|(u, _)| *u == uri(ROOT)).unwrap();
    assert_eq!(root.1.len(), 1, "{:#?}", root.1);
    let diagnostic = &root.1[0];
    assert_eq!(diagnostic["severity"], 1);
    assert_eq!(diagnostic["code"], "context");
    assert!(
        diagnostic["message"]
            .as_str()
            .unwrap()
            .starts_with("`tick` runs from #[tick] without a player, but it calls `grant`")
    );
    assert_eq!(
        diagnostic["relatedInformation"].as_array().unwrap().len(),
        2
    );

    client.shutdown();
}

#[test]
fn uris_round_trip_through_paths() {
    let path = std::path::Path::new("/tmp/My Packs/pack.sand");
    let uri = sand_lsp::path_to_uri(path).unwrap();
    assert_eq!(uri.as_str(), "file:///tmp/My%20Packs/pack.sand");
    assert_eq!(sand_lsp::uri_to_path(&uri).unwrap(), path);
}

#[test]
fn files_outside_the_module_tree_are_still_analysed() {
    let mut client = Client::start();
    client.open(ROOT, ROOT_TEXT);
    let _ = client.diagnostics(1);
    // `orphan.sand` sits next to the root but no `mod` declares it.
    let orphan = "/sand-lsp-test/orphan.sand";
    client.open(orphan, "fn f() { give_item(Item::Dirt); }\n");
    let published = client.diagnostics(1);
    let (uri, diagnostics) = &published[0];
    assert_eq!(uri, &format!("file://{orphan}"));
    assert!(
        diagnostics.iter().any(|d| d["message"]
            .as_str()
            .unwrap()
            .starts_with("the package root must declare a namespace\nhelp: ")),
        "{diagnostics:#?}"
    );
    let hover = client.request(
        "textDocument/hover",
        json!({"textDocument": {"uri": format!("file://{orphan}")}, "position": {"line": 0, "character": 10}}),
    );
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Give items")
    );
    client.shutdown();
}
