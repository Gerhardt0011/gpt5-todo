use crate::domain::repository::TodoRepository;
use crate::domain::todo::{CreateTodo, Todo, TodoId, UpdateTodo};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TodoService: Send + Sync + 'static {
    async fn create(&self, input: CreateTodo) -> Result<Todo>;
    async fn get(&self, id: TodoId) -> Result<Option<Todo>>;
    async fn list(&self) -> Result<Vec<Todo>>;
    async fn update(&self, id: TodoId, input: UpdateTodo) -> Result<Option<Todo>>;
    async fn delete(&self, id: TodoId) -> Result<bool>;
}

#[derive(Clone)]
pub struct TodoServiceImpl<R: TodoRepository> {
    repo: R,
}

impl<R: TodoRepository> TodoServiceImpl<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
}

#[async_trait]
impl<R: TodoRepository> TodoService for TodoServiceImpl<R> {
    async fn create(&self, input: CreateTodo) -> Result<Todo> { self.repo.create(input).await }
    async fn get(&self, id: TodoId) -> Result<Option<Todo>> { self.repo.get(id).await }
    async fn list(&self) -> Result<Vec<Todo>> { self.repo.list().await }
    async fn update(&self, id: TodoId, input: UpdateTodo) -> Result<Option<Todo>> { self.repo.update(id, input).await }
    async fn delete(&self, id: TodoId) -> Result<bool> { self.repo.delete(id).await }
}
