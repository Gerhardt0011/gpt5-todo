#[cfg(test)]
mod tests {
    use super::super::todo_service::{TodoService, TodoServiceImpl};
    use crate::domain::{repository::TodoRepository, todo::{CreateTodo, Todo, TodoId, TodoStatus, UpdateTodo}};
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;

    #[derive(Clone, Default)]
    struct InMemoryRepo {
        items: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Todo>>>,
    }

    #[async_trait]
    impl TodoRepository for InMemoryRepo {
        async fn init(&self) -> Result<()> { Ok(()) }
        async fn create(&self, input: CreateTodo) -> Result<Todo> {
            let now = Utc::now();
            let id = TodoId(uuid::Uuid::new_v4());
            let todo = Todo { id: id.clone(), title: input.title, description: input.description, status: TodoStatus::Pending, created_at: now, updated_at: now };
            self.items.lock().unwrap().insert(id.0.to_string(), todo.clone());
            Ok(todo)
        }
        async fn get(&self, id: TodoId) -> Result<Option<Todo>> { Ok(self.items.lock().unwrap().get(&id.0.to_string()).cloned()) }
        async fn list(&self) -> Result<Vec<Todo>> { Ok(self.items.lock().unwrap().values().cloned().collect()) }
        async fn update(&self, id: TodoId, input: UpdateTodo) -> Result<Option<Todo>> {
            let mut map = self.items.lock().unwrap();
            let Some(mut todo) = map.get(&id.0.to_string()).cloned() else { return Ok(None) };
            if let Some(t) = input.title { todo.title = t; }
            if let Some(d) = input.description { todo.description = Some(d); }
            if let Some(s) = input.status { todo.status = s; }
            todo.updated_at = Utc::now();
            map.insert(id.0.to_string(), todo.clone());
            Ok(Some(todo))
        }
        async fn delete(&self, id: TodoId) -> Result<bool> { Ok(self.items.lock().unwrap().remove(&id.0.to_string()).is_some()) }
    }

    #[tokio::test]
    async fn unit_create_and_get() {
        let repo = InMemoryRepo::default();
        let service = TodoServiceImpl::new(repo);
        let created = service.create(CreateTodo { title: "X".into(), description: None }).await.unwrap();
        assert_eq!(created.title, "X");
        let got = service.get(created.id.clone()).await.unwrap().unwrap();
        assert_eq!(got.id, created.id);
    }
}
