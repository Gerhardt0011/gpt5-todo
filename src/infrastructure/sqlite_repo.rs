use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::{SqlitePoolOptions, SqliteRow}, Pool, Row, Sqlite};
use uuid::Uuid;

use crate::domain::{
    repository::TodoRepository,
    todo::{CreateTodo, Todo, TodoId, TodoStatus, UpdateTodo},
};

#[derive(Clone)]
pub struct SqliteTodoRepository {
    pool: Arc<Pool<Sqlite>>,
}

impl SqliteTodoRepository {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool: Arc::new(pool) })
    }
}

#[async_trait]
impl TodoRepository for SqliteTodoRepository {
    async fn init(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn create(&self, input: CreateTodo) -> Result<Todo> {
        let now = Utc::now();
        let id = TodoId(Uuid::new_v4());
        let status = TodoStatus::Pending;
        sqlx::query(
            "INSERT INTO todos (id, title, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(id.0.to_string())
        .bind(&input.title)
        .bind(&input.description)
        .bind(match status { TodoStatus::Pending => "pending", TodoStatus::Done => "done" })
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&*self.pool)
        .await?;
        Ok(Todo { id, title: input.title, description: input.description, status, created_at: now, updated_at: now })
    }

    async fn get(&self, id: TodoId) -> Result<Option<Todo>> {
        let row = sqlx::query("SELECT id, title, description, status, created_at, updated_at FROM todos WHERE id = ?1")
            .bind(id.0.to_string())
            .fetch_optional(&*self.pool)
            .await?;
        Ok(row.map(row_to_todo))
    }

    async fn list(&self) -> Result<Vec<Todo>> {
        let rows = sqlx::query("SELECT id, title, description, status, created_at, updated_at FROM todos ORDER BY created_at DESC")
            .fetch_all(&*self.pool)
            .await?;
        Ok(rows.into_iter().map(row_to_todo).collect())
    }

    async fn update(&self, id: TodoId, input: UpdateTodo) -> Result<Option<Todo>> {
        // Fetch existing
        let existing = self.get(id.clone()).await?;
        let Some(mut todo) = existing else { return Ok(None) };

        if let Some(t) = input.title { todo.title = t; }
        if let Some(d) = input.description { todo.description = Some(d); }
        if let Some(s) = input.status { todo.status = s; }
        todo.updated_at = Utc::now();

        sqlx::query("UPDATE todos SET title = ?2, description = ?3, status = ?4, updated_at = ?5 WHERE id = ?1")
            .bind(todo.id.0.to_string())
            .bind(&todo.title)
            .bind(&todo.description)
            .bind(match todo.status { TodoStatus::Pending => "pending", TodoStatus::Done => "done" })
            .bind(todo.updated_at.to_rfc3339())
            .execute(&*self.pool)
            .await?;

        Ok(Some(todo))
    }

    async fn delete(&self, id: TodoId) -> Result<bool> {
        let result = sqlx::query("DELETE FROM todos WHERE id = ?1")
            .bind(id.0.to_string())
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

fn row_to_todo(row: SqliteRow) -> Todo {
    let id_str: String = row.get("id");
    let title: String = row.get("title");
    let description: Option<String> = row.get("description");
    let status_str: String = row.get("status");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");

    let status = match status_str.as_str() { "pending" => TodoStatus::Pending, "done" => TodoStatus::Done, _ => TodoStatus::Pending };
    let created_at = DateTime::parse_from_rfc3339(&created_at_str).unwrap().with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str).unwrap().with_timezone(&Utc);

    Todo {
        id: TodoId(Uuid::parse_str(&id_str).unwrap()),
        title,
        description,
        status,
        created_at,
        updated_at,
    }
}
