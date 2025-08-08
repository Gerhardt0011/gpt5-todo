use async_trait::async_trait;
use super::todo::{Todo, TodoId, CreateTodo, UpdateTodo};

#[async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    async fn init(&self) -> anyhow::Result<()>;
    async fn create(&self, input: CreateTodo) -> anyhow::Result<Todo>;
    async fn get(&self, id: TodoId) -> anyhow::Result<Option<Todo>>;
    async fn list(&self) -> anyhow::Result<Vec<Todo>>;
    async fn update(&self, id: TodoId, input: UpdateTodo) -> anyhow::Result<Option<Todo>>;
    async fn delete(&self, id: TodoId) -> anyhow::Result<bool>;
}
