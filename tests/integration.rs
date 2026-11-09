use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use practica::{JsonStore, create_app};
use serde_json::{Value, json};
use tempfile::tempdir;
use tower::ServiceExt;

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

#[tokio::test]
async fn health_endpoint() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn full_crud_flow() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    // create
    let req = Request::builder()
        .method("POST")
        .uri("/todos")
        .header("content-type", "application/json")
        .body(Body::from(json!({"title": "buy milk"}).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = json_body(resp).await;
    let id = created["id"].as_u64().unwrap();
    assert_eq!(created["title"], "buy milk");
    assert_eq!(created["completed"], false);

    // list
    let req = Request::builder()
        .uri("/todos")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = json_body(resp).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // get
    let req = Request::builder()
        .uri(format!("/todos/{}", id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // update
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/todos/{}", id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"title": "buy oat milk", "completed": true}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = json_body(resp).await;
    assert_eq!(updated["title"], "buy oat milk");
    assert_eq!(updated["completed"], true);

    // delete
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/todos/{}", id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // get after delete -> 404
    let req = Request::builder()
        .uri(format!("/todos/{}", id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_rejects_empty_title() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    let req = Request::builder()
        .method("POST")
        .uri("/todos")
        .header("content-type", "application/json")
        .body(Body::from(json!({"title": "   "}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_missing_returns_404() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    let req = Request::builder()
        .uri("/todos/999")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_missing_returns_404() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    let req = Request::builder()
        .method("PUT")
        .uri("/todos/999")
        .header("content-type", "application/json")
        .body(Body::from(json!({"completed": true}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_missing_returns_404() {
    let dir = tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("db.json")).unwrap();
    let app = create_app(store);

    let req = Request::builder()
        .method("DELETE")
        .uri("/todos/999")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
