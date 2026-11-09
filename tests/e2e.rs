use serde_json::json;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn e2e_crud_over_http() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("e2e.json");
    let port = 20000 + (std::process::id() % 10000) as u16;

    let bin = env!("CARGO_BIN_EXE_practica");
    let mut child = Command::new(bin)
        .env("TODO_DB_PATH", &db_path)
        .env("PORT", port.to_string())
        .spawn()
        .expect("failed to spawn server");

    let client = reqwest::blocking::Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    let mut ready = false;
    for _ in 0..50 {
        if client.get(format!("{}/health", base)).send().is_ok() {
            ready = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    assert!(ready, "server did not become ready");

    // create
    let resp = client
        .post(format!("{}/todos", base))
        .json(&json!({"title": "e2e task"}))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 201);
    let created: serde_json::Value = resp.json().unwrap();
    let id = created["id"].as_u64().unwrap();
    assert_eq!(created["title"], "e2e task");

    // list
    let resp = client.get(format!("{}/todos", base)).send().unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);

    // get
    let resp = client.get(format!("{}/todos/{}", base, id)).send().unwrap();
    assert_eq!(resp.status(), 200);

    // update
    let resp = client
        .put(format!("{}/todos/{}", base, id))
        .json(&json!({"completed": true}))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: serde_json::Value = resp.json().unwrap();
    assert_eq!(updated["completed"], true);

    // delete
    let resp = client
        .delete(format!("{}/todos/{}", base, id))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 204);

    // get after delete
    let resp = client.get(format!("{}/todos/{}", base, id)).send().unwrap();
    assert_eq!(resp.status(), 404);

    let _ = child.kill();
    let _ = child.wait();
}
