use api::{application::todo_service::TodoServiceImpl, http::routing, http::routing::todos, infrastructure::sqlite_repo::SqliteTodoRepository};
use api::domain::repository::TodoRepository;
use axum::body::to_bytes;
use axum::Router;
use serde_json::json;

#[tokio::test]
async fn acceptance_create_list_get_update_delete() {
    // use in-memory sqlite for tests
    let repo = SqliteTodoRepository::connect("sqlite::memory:").await.unwrap();
    repo.init().await.unwrap();
    let service = TodoServiceImpl::new(repo);
    let app: Router = routing::app(todos::router(todos::AppState { service }));

    // create
    let payload = json!({ "title": "Test", "description": "First" });
    let res = request(&app, "POST", "/todos", Some(payload)).await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(res.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    let id = body.get("id").unwrap().as_str().unwrap().to_string();

    // list
    let res = request(&app, "GET", "/todos", None).await;
    assert_eq!(res.status(), 200);

    // get
    let res = request(&app, "GET", &format!("/todos/{}", id), None).await;
    assert_eq!(res.status(), 200);

    // update
    let res = request(&app, "PUT", &format!("/todos/{}", id), Some(json!({"status":"done"}))).await;
    assert_eq!(res.status(), 200);

    // delete
    let res = request(&app, "DELETE", &format!("/todos/{}", id), None).await;
    assert_eq!(res.status(), 204);

    // get 404
    let res = request(&app, "GET", &format!("/todos/{}", id), None).await;
    assert_eq!(res.status(), 404);
}

async fn request(app: &Router, method: &str, path: &str, body: Option<serde_json::Value>) -> hyper::Response<axum::body::Body> {
    use axum::body::Body;
    use axum::http::{Request, Method};
    use tower::ServiceExt;

    let req = Request::builder().method(Method::from_bytes(method.as_bytes()).unwrap()).uri(path);
    let req = match body {
        Some(json) => req.header("content-type", "application/json").body(Body::from(json.to_string())).unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    app.clone().oneshot(req).await.unwrap()
}
