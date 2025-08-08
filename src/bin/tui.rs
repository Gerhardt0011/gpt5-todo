use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, List, ListItem, Paragraph, ListState}, layout::{Layout, Constraint, Direction}, style::{Style, Modifier, Color}};

use api::{application::todo_service::{TodoService, TodoServiceImpl}, domain::{repository::TodoRepository, todo::{CreateTodo, TodoStatus}}, infrastructure::sqlite_repo::SqliteTodoRepository};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todos.db".to_string());
    prepare_sqlite_file(&database_url)?;
    let repo = SqliteTodoRepository::connect(&database_url).await?;
    repo.init().await?;
    let service = TodoServiceImpl::new(repo);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, service).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    res
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { View, Create, Edit }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter { All, Pending, Done }

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveField { Title, Description }

struct ListEntry {
    id: uuid::Uuid,
    status: TodoStatus,
    title: String,
    description: Option<String>,
}

struct App<R: TodoRepository> {
    service: TodoServiceImpl<R>,
    items: Vec<ListEntry>,
    selected: usize,
    last_tick: Instant,
    mode: Mode,
    list_state: ListState,
    filter: Filter,
    filtered_indices: Vec<usize>,
    field: ActiveField,
    draft_title: String,
    draft_desc: String,
}

impl<R: TodoRepository> App<R> {
    async fn load(&mut self) -> Result<()> {
        let todos = self.service.list().await?;
        self.items = todos
            .into_iter()
            .map(|t| ListEntry { id: t.id.0, status: t.status, title: t.title, description: t.description })
            .collect();
        self.recompute_filtered();
        Ok(())
    }

    fn recompute_filtered(&mut self) {
        self.filtered_indices.clear();
        for (i, e) in self.items.iter().enumerate() {
            let include = match self.filter {
                Filter::All => true,
                Filter::Pending => matches!(e.status, TodoStatus::Pending),
                Filter::Done => matches!(e.status, TodoStatus::Done),
            };
            if include { self.filtered_indices.push(i); }
        }
        // Clamp selection within filtered bounds
        let len = self.filtered_indices.len();
        if len == 0 { self.selected = 0; self.list_state.select(None); }
        else { if self.selected >= len { self.selected = len - 1; } self.list_state.select(Some(self.selected)); }
    }
}

async fn run_app<R: TodoRepository>(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, service: TodoServiceImpl<R>) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut app = App { service, items: vec![], selected: 0, last_tick: Instant::now(), mode: Mode::View, list_state: ListState::default(), filter: Filter::All, filtered_indices: Vec::new(), field: ActiveField::Title, draft_title: String::new(), draft_desc: String::new() };
    app.load().await?;

    loop {
        terminal.draw(|f| {
        let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
            Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let header = Paragraph::new("Todos (Enter: toggle, n: new, e: edit, d: delete, f: filter, q: quit)  |  New/Edit: type title, Enter to save, Esc to cancel")
                .block(Block::default().borders(Borders::ALL).title("api-tui"));
            f.render_widget(header, chunks[0]);

            let middle = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[1]);

            let list_items: Vec<ListItem> = app.filtered_indices.iter().filter_map(|&idx| app.items.get(idx)).map(|e| {
                let mark = match e.status { TodoStatus::Pending => "[ ]", TodoStatus::Done => "[x]" };
                ListItem::new(format!("{} {}", mark, e.title))
            }).collect();
            // Keep list_state selection in sync with current index
            if app.filtered_indices.is_empty() { app.list_state.select(None); } else { app.list_state.select(Some(app.selected)); }
            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(format!("items [{}] (highlighted = target for Enter/d/e)", match app.filter { Filter::All => "All", Filter::Pending => "Pending", Filter::Done => "Done" })))
                .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::REVERSED))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, middle[0], &mut app.list_state);

            // Details pane for selected item (shows description)
            let detail = if let Some(&idx) = app.filtered_indices.get(app.selected) {
                if let Some(e) = app.items.get(idx) {
                    let desc = e.description.clone().unwrap_or_else(|| "(no description)".to_string());
                    format!("Title:\n{}\n\nStatus: {}\n\nDescription:\n{}", e.title, match e.status { TodoStatus::Pending => "Pending", TodoStatus::Done => "Done" }, desc)
                } else { "".to_string() }
            } else { "".to_string() };
            let details = Paragraph::new(detail)
                .block(Block::default().borders(Borders::ALL).title("details"));
            f.render_widget(details, middle[1]);

            let footer_text = match app.mode {
                Mode::View => format!("DATABASE_URL={}  |  Filter=[{}]", std::env::var("DATABASE_URL").unwrap_or_default(), match app.filter { Filter::All => "All", Filter::Pending => "Pending", Filter::Done => "Done" }),
                Mode::Create => format!("Create — {}: {}_  |  (Tab to switch, Enter to save, Esc to cancel)", match app.field { ActiveField::Title => "Title", ActiveField::Description => "Desc" }, match app.field { ActiveField::Title => &app.draft_title, ActiveField::Description => &app.draft_desc }),
                Mode::Edit => format!("Edit — {}: {}_  |  (Tab to switch, Enter to save, Esc to cancel)", match app.field { ActiveField::Title => "Title", ActiveField::Description => "Desc" }, match app.field { ActiveField::Title => &app.draft_title, ActiveField::Description => &app.draft_desc }),
            };
            let footer = Paragraph::new(footer_text)
                .block(Block::default().borders(Borders::ALL).title(match app.mode { Mode::View => "info", Mode::Create => "create", Mode::Edit => "edit" }));
            f.render_widget(footer, chunks[2]);
        })?;

        let timeout = tick_rate.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Only act on key presses; ignore repeats and releases to prevent duplicate input
                if key.kind != KeyEventKind::Press { continue; }
                match app.mode {
                    Mode::View => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => { if app.selected > 0 { app.selected -= 1; } }
                        KeyCode::Down => { let len = app.filtered_indices.len(); if app.selected + 1 < len { app.selected += 1; } }
                        KeyCode::Enter => {
                            if let Some(entry) = app.items.get(app.selected) {
                                let new_status = match entry.status { TodoStatus::Pending => TodoStatus::Done, TodoStatus::Done => TodoStatus::Pending };
                                let _ = app.service.update(api::domain::todo::TodoId(entry.id), api::domain::todo::UpdateTodo { title: None, description: None, status: Some(new_status) }).await;
                                app.load().await?;
                            }
                        }
                        KeyCode::Char('n') => {
                            app.mode = Mode::Create;
                            app.field = ActiveField::Title;
                            app.draft_title.clear();
                            app.draft_desc.clear();
                        }
                        KeyCode::Char('e') => {
                            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                                if let Some(entry) = app.items.get(idx) { 
                                    app.mode = Mode::Edit; 
                                    app.field = ActiveField::Title;
                                    app.draft_title = entry.title.clone();
                                    app.draft_desc = entry.description.clone().unwrap_or_default();
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                                if let Some(entry) = app.items.get(idx) {
                                let _ = app.service.delete(api::domain::todo::TodoId(entry.id)).await;
                                if app.selected > 0 { app.selected -= 1; }
                                app.load().await?;
                                }
                            }
                        }
                        KeyCode::Char('f') => {
                            app.filter = match app.filter { Filter::All => Filter::Pending, Filter::Pending => Filter::Done, Filter::Done => Filter::All };
                            app.recompute_filtered();
                        }
                        _ => {}
                    },
                    Mode::Create => match key.code {
                        KeyCode::Esc => { app.mode = Mode::View; app.draft_title.clear(); app.draft_desc.clear(); }
                        KeyCode::Enter => {
                            let title = app.draft_title.trim();
                            let desc = app.draft_desc.trim();
                            if !title.is_empty() {
                                let desc_opt = if desc.is_empty() { None } else { Some(desc.to_string()) };
                                let _ = app.service.create(CreateTodo { title: title.to_string(), description: desc_opt }).await;
                            }
                            app.mode = Mode::View;
                            app.draft_title.clear();
                            app.draft_desc.clear();
                            app.load().await?;
                        }
                        KeyCode::Backspace => { match app.field { ActiveField::Title => { app.draft_title.pop(); }, ActiveField::Description => { app.draft_desc.pop(); } } }
                        KeyCode::Char(c) => { match app.field { ActiveField::Title => app.draft_title.push(c), ActiveField::Description => app.draft_desc.push(c) } }
                        KeyCode::Tab => { app.field = match app.field { ActiveField::Title => ActiveField::Description, ActiveField::Description => ActiveField::Title }; }
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => { /* ignore nav in input */ }
                        _ => {}
                    },
                    Mode::Edit => match key.code {
                        KeyCode::Esc => { app.mode = Mode::View; app.draft_title.clear(); app.draft_desc.clear(); }
                        KeyCode::Enter => {
                            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                                if let Some(entry) = app.items.get(idx) {
                                    let title = app.draft_title.trim().to_string();
                                    let desc = app.draft_desc.trim().to_string();
                                    let title_opt = if title.is_empty() { None } else { Some(title) };
                                    let desc_opt = if desc.is_empty() { Some(String::new()) } else { Some(desc) };
                                    let _ = app.service.update(api::domain::todo::TodoId(entry.id), api::domain::todo::UpdateTodo { title: title_opt, description: desc_opt, status: None }).await;
                                }
                            }
                            app.mode = Mode::View;
                            app.draft_title.clear();
                            app.draft_desc.clear();
                            app.load().await?;
                        }
                        KeyCode::Backspace => { match app.field { ActiveField::Title => { app.draft_title.pop(); }, ActiveField::Description => { app.draft_desc.pop(); } } }
                        KeyCode::Char(c) => { match app.field { ActiveField::Title => app.draft_title.push(c), ActiveField::Description => app.draft_desc.push(c) } }
                        KeyCode::Tab => { app.field = match app.field { ActiveField::Title => ActiveField::Description, ActiveField::Description => ActiveField::Title }; }
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => { /* ignore nav in input */ }
                        _ => {}
                    },
                }
            }
        }
        if app.last_tick.elapsed() >= tick_rate {
            app.last_tick = Instant::now();
        }
    }
    Ok(())
}

fn prepare_sqlite_file(database_url: &str) -> anyhow::Result<()> {
    if database_url.starts_with("sqlite::memory:") { return Ok(()); }
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        let path = if cfg!(windows) && path.len() >= 3 && path.as_bytes()[0] == b'/' && path.as_bytes()[2] == b':' { &path[1..] } else { path };
        use std::{fs, path::Path, fs::OpenOptions};
        let p = Path::new(path);
        if let Some(parent) = p.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
        if !p.exists() { let _ = OpenOptions::new().create(true).append(true).open(p)?; }
    }
    Ok(())
}
