use crate::common::delegate::{BackoffConfig, ChunkSize, UploadDelegateConfig};
use crate::common::drive_file;
use crate::common::file_info;
use crate::common::file_tree;
use crate::common::hub_helper;
use crate::common::id_gen::IdGen;
use crate::common::md5_writer::Md5Writer;
use crate::files;
use crate::files::info::DisplayConfig;
use crate::files::list::{ListFilesConfig, ListQuery, ListSortOrder};
use crate::files::mkdir;
use crate::files::upload;
use crate::hub::Hub;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use human_bytes::human_bytes;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::runtime::Handle;

const HELP_LABELS: [(&str, &str); 7] = [
    ("Enter/→  : open", "open"),
    ("←/b  : back", "back"),
    ("d  : download", "download"),
    ("u  : upload menu", "upload"),
    ("x  : delete", "delete"),
    ("r  : refresh", "refresh"),
    ("q  : quit", "quit"),
];

pub async fn navigate() -> Result<(), Error> {
    let handle = Handle::current();
    let result = tokio::task::spawn_blocking(move || run_app(handle)).await;
    match result {
        Ok(inner) => inner,
        Err(err) => Err(Error::Join(err)),
    }
}

fn run_app(handle: Handle) -> Result<(), Error> {
    enable_raw_mode().map_err(Error::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(Error::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(Error::Io)?;

    let result = run_loop(&mut terminal, handle);

    disable_raw_mode().map_err(Error::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(Error::Io)?;
    terminal.show_cursor().map_err(Error::Io)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, handle: Handle) -> Result<(), Error> {
    let hub = handle
        .block_on(hub_helper::get_hub())
        .map_err(Error::Hub)?;
    let mut app = App::new(hub);
    app.reload(&handle)?;

    loop {
        app.tick();
        terminal.draw(|frame| draw_ui(frame, &app)).map_err(Error::Io)?;

        if app.should_exit() {
            break;
        }

        if !event::poll(Duration::from_millis(250)).map_err(Error::Io)? {
            continue;
        }

        if let Event::Key(key) = event::read().map_err(Error::Io)? {
            if handle_key_event(&mut app, key, &handle)? {
                break;
            }
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent, handle: &Handle) -> Result<bool, Error> {
    match app.input_mode {
        InputMode::Normal => handle_normal_key(app, key, handle),
        InputMode::DownloadDestination => handle_input_key(app, key, handle),
        InputMode::UploadPicker => handle_upload_picker_key(app, key, handle),
        InputMode::DeleteConfirm => handle_delete_confirm_key(app, key, handle),
        InputMode::QuitConfirm => handle_quit_confirm_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent, handle: &Handle) -> Result<bool, Error> {
    match key.code {
        KeyCode::Char('q') => {
            if app.can_quit() {
                return Ok(true);
            }
            app.start_quit_confirm();
        }
        KeyCode::Char('r') => {
            app.reload(handle)?;
        }
        KeyCode::Char('b') | KeyCode::Left => {
            app.go_back(handle)?;
        }
        KeyCode::Char('d') => {
            app.start_input(InputMode::DownloadDestination, "Download destination (dir)");
        }
        KeyCode::Char('u') => {
            app.start_upload_picker();
        }
        KeyCode::Char('x') => {
            app.start_delete_confirm();
        }
        KeyCode::Delete => {
            app.start_delete_confirm();
        }
        KeyCode::Up => {
            app.select_previous();
        }
        KeyCode::Down => {
            app.select_next();
        }
        KeyCode::Enter | KeyCode::Right => {
            app.open_selected(handle)?;
        }
        _ => {}
    }

    Ok(false)
}

fn handle_input_key(app: &mut App, key: KeyEvent, handle: &Handle) -> Result<bool, Error> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_input("Cancelled");
        }
        KeyCode::Enter => {
            let input = app.input.clone();
            let mode = app.input_mode;
            app.input.clear();
            app.input_mode = InputMode::Normal;

            match mode {
                InputMode::DownloadDestination => {
                    let destination = if input.trim().is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(input.trim()))
                    };
                    if let Err(err) = app.download_selected(handle, destination) {
                        app.status = format!("Error: {}", err);
                    } else {
                        app.status = "Download completed".to_string();
                    }
                }
                InputMode::Normal | InputMode::UploadPicker | InputMode::DeleteConfirm | InputMode::QuitConfirm => {}
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(ch) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }
            app.input.push(ch);
        }
        _ => {}
    }

    Ok(false)
}

fn handle_upload_picker_key(
    app: &mut App,
    key: KeyEvent,
    handle: &Handle,
) -> Result<bool, Error> {
    let picker = match app.upload_picker.as_mut() {
        Some(picker) => picker,
        None => {
            app.cancel_input("Upload picker closed");
            return Ok(false);
        }
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.cancel_input("Upload cancelled");
        }
        KeyCode::Char('b') | KeyCode::Left => {
            if let Some(parent) = picker.current_dir.parent().map(|p| p.to_path_buf()) {
                picker.current_dir = parent;
                picker.reload().map_err(Error::Io)?;
            }
        }
        KeyCode::Right => {
            if let Some(entry) = picker.entries.get(picker.selected) {
                if entry.is_dir {
                    picker.current_dir = entry.path.clone();
                    picker.reload().map_err(Error::Io)?;
                }
            }
        }
        KeyCode::Up => {
            if !picker.entries.is_empty() {
                if picker.selected == 0 {
                    picker.selected = picker.entries.len() - 1;
                } else {
                    picker.selected -= 1;
                }
            }
        }
        KeyCode::Down => {
            if !picker.entries.is_empty() {
                picker.selected = (picker.selected + 1) % picker.entries.len();
            }
        }
        KeyCode::Enter => {
            if let Some(entry) = picker.entries.get(picker.selected) {
                if entry.is_parent {
                    picker.current_dir = entry.path.clone();
                    picker.reload().map_err(Error::Io)?;
                } else {
                    picker.selected_path = Some(entry.path.clone());
                    app.status = format!("Selected {}", entry.name);
                }
            }
        }
        KeyCode::Char('u') => {
            let selection = picker
                .selected_path
                .clone()
                .or_else(|| picker.entries.get(picker.selected).map(|e| e.path.clone()));
            match selection {
                Some(path) => {
                    app.input_mode = InputMode::Normal;
                    app.upload_picker = None;
                    app.start_upload_job(handle, path)?;
                }
                None => {
                    app.status = "No selection to upload".to_string();
                }
            }
        }
        _ => {}
    }

    Ok(false)
}

fn handle_delete_confirm_key(
    app: &mut App,
    key: KeyEvent,
    handle: &Handle,
) -> Result<bool, Error> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.pending_delete = None;
            app.input_mode = InputMode::Normal;
            app.status = "Delete cancelled".to_string();
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(item) = app.pending_delete.clone() {
                app.pending_delete = None;
                app.input_mode = InputMode::Normal;
                if let Err(err) = app.delete_item(handle, item) {
                    app.status = format!("Delete failed: {}", err);
                } else {
                    app.status = "Delete completed".to_string();
                    app.reload(handle)?;
                }
            } else {
                app.input_mode = InputMode::Normal;
                app.status = "Nothing to delete".to_string();
            }
        }
        _ => {}
    }

    Ok(false)
}

fn handle_quit_confirm_key(app: &mut App, key: KeyEvent) -> Result<bool, Error> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.input_mode = InputMode::Normal;
            app.status = "Quit cancelled".to_string();
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.input_mode = InputMode::Normal;
            app.request_exit();
        }
        _ => {}
    }
    Ok(false)
}

fn draw_ui(frame: &mut ratatui::Frame<'_>, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.size());

    let header = Paragraph::new(Line::from(vec![
        Span::raw("Folder: "),
        Span::styled(
            app.current_folder_name.as_str(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(header, layout[0]);

    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            let label = if item.is_parent {
                item.name.clone()
            } else if item.is_folder {
                format!("[DIR] {}", item.name)
            } else if let Some(size) = item.size {
                let formatted =
                    files::info::format_bytes(size, &DisplayConfig::default());
                format!("{} ({})", item.name, formatted)
            } else {
                item.name.clone()
            };
            ListItem::new(Line::from(label))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Drive")
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    if !app.items.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, layout[1], &mut state);

    let footer_text = match app.input_mode {
        InputMode::Normal => {
            let status = app.render_status();
            let mut spans = vec![Span::raw(status), Span::raw(" | ")];
            for (index, (label, tag)) in HELP_LABELS.iter().enumerate() {
                let color = match *tag {
                    "open" => Color::Cyan,
                    "back" => Color::Magenta,
                    "download" => Color::Yellow,
                    "upload" => Color::Green,
                    "delete" => Color::Red,
                    "refresh" => Color::Blue,
                    "quit" => Color::Red,
                    _ => Color::White,
                };
                if index > 0 {
                    spans.push(Span::raw("  "));
                }
                spans.push(Span::styled(*label, Style::default().fg(color)));
            }
            Line::from(spans)
        }
        InputMode::DownloadDestination => {
            let current_dir = std::env::current_dir()
                .ok()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            Line::from(vec![
                Span::raw(format!("Download to dir (empty = {}): ", current_dir)),
                Span::styled(app.input.as_str(), Style::default().add_modifier(Modifier::BOLD)),
            ])
        }
        InputMode::UploadPicker => {
            let selected = app
                .upload_picker
                .as_ref()
                .and_then(|picker| picker.selected_path.clone())
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_string());
            let mut spans = vec![
                Span::raw("Upload picker  "),
                Span::styled("Enter: select", Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled("→: open dir", Style::default().fg(Color::Blue)),
                Span::raw("  "),
                Span::styled("←/b: up", Style::default().fg(Color::Magenta)),
                Span::raw("  "),
                Span::styled("u: upload", Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled("Esc: cancel", Style::default().fg(Color::Red)),
                Span::raw(" | Selected: "),
                Span::styled(selected, Style::default().add_modifier(Modifier::BOLD)),
            ];
            if app.blink_on {
                if let Some(picker) = &app.upload_picker {
                    if picker.selected_path.is_some() {
                        spans.push(Span::raw("  "));
                        spans.push(Span::styled(
                            "press u to start uploading",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                }
            }
            Line::from(spans)
        }
        InputMode::DeleteConfirm => Line::from(vec![Span::raw("Confirm delete...")]),
        InputMode::QuitConfirm => Line::from(vec![Span::raw("Confirm quit...")]),
    };

    let footer = Paragraph::new(footer_text);
    frame.render_widget(footer, layout[2]);

    if app.input_mode == InputMode::UploadPicker {
        draw_upload_picker(frame, app);
    }
    if app.input_mode == InputMode::DeleteConfirm {
        draw_delete_confirm(frame, app);
    }
    if app.input_mode == InputMode::QuitConfirm {
        draw_quit_confirm(frame, app);
    }
}

fn draw_upload_picker(frame: &mut ratatui::Frame<'_>, app: &App) {
    let picker = match &app.upload_picker {
        Some(picker) => picker,
        None => return,
    };
    let area = centered_rect(80, 70, frame.size());
    frame.render_widget(Clear, area);
    let entries: Vec<ListItem> = picker
        .entries
        .iter()
        .map(|entry| {
            let label = if entry.is_parent {
                "/..".to_string()
            } else if entry.is_dir {
                format!("[DIR] {}", entry.name)
            } else {
                entry.name.clone()
            };
            ListItem::new(Line::from(label))
        })
        .collect();

    let list = List::new(entries)
        .block(
            Block::default()
                .title(format!(
                    "Upload from {}",
                    picker.current_dir.display()
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    if !picker.entries.is_empty() {
        state.select(Some(picker.selected));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_delete_confirm(frame: &mut ratatui::Frame<'_>, app: &App) {
    let item = match &app.pending_delete {
        Some(item) => item,
        None => return,
    };
    let area = centered_rect(50, 30, frame.size());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(vec![
            Span::raw("Delete "),
            Span::styled(
                item.name.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from("Are you sure?"),
        Line::from(vec![
            Span::styled("[y] Yes", Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled("[n] No", Style::default().fg(Color::Green)),
        ]),
    ];

    let block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_quit_confirm(frame: &mut ratatui::Frame<'_>, app: &App) {
    if !app.has_active_transfer() {
        return;
    }
    let area = centered_rect(50, 30, frame.size());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from("There are active transfers."),
        Line::from("Are you sure you want to quit?"),
        Line::from(vec![
            Span::styled("[y] Yes", Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled("[n] No", Style::default().fg(Color::Green)),
        ]),
    ];
    let block = Block::default()
        .title("Confirm Quit")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[derive(Debug, Clone)]
struct DriveItem {
    id: String,
    name: String,
    is_folder: bool,
    size: Option<i64>,
    is_parent: bool,
}

#[derive(Debug, Clone)]
struct FolderState {
    id: Option<String>,
    name: String,
}

#[derive(Debug, Clone)]
struct LocalEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_parent: bool,
}

struct UploadPicker {
    current_dir: PathBuf,
    entries: Vec<LocalEntry>,
    selected: usize,
    selected_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum InputMode {
    Normal,
    DownloadDestination,
    UploadPicker,
    DeleteConfirm,
    QuitConfirm,
}

struct App {
    hub: Hub,
    items: Vec<DriveItem>,
    selected: usize,
    folder_stack: Vec<FolderState>,
    current_folder_id: Option<String>,
    current_folder_name: String,
    status: String,
    input_mode: InputMode,
    input: String,
    download_job: Option<DownloadJob>,
    upload_picker: Option<UploadPicker>,
    upload_job: Option<UploadJob>,
    blink_on: bool,
    last_blink: Instant,
    pending_delete: Option<DriveItem>,
    exit_requested: bool,
}

impl App {
    fn new(hub: Hub) -> Self {
        Self {
            hub,
            items: Vec::new(),
            selected: 0,
            folder_stack: Vec::new(),
            current_folder_id: None,
            current_folder_name: "root".to_string(),
            status: "Ready".to_string(),
            input_mode: InputMode::Normal,
            input: String::new(),
            download_job: None,
            upload_picker: None,
            upload_job: None,
            blink_on: true,
            last_blink: Instant::now(),
            pending_delete: None,
            exit_requested: false,
        }
    }

    fn start_input(&mut self, mode: InputMode, status: &str) {
        self.input_mode = mode;
        self.input.clear();
        self.status = status.to_string();
    }

    fn cancel_input(&mut self, status: &str) {
        self.input_mode = InputMode::Normal;
        self.input.clear();
        self.status = status.to_string();
        self.upload_picker = None;
    }

    fn start_upload_picker(&mut self) {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        match UploadPicker::from_dir(current_dir) {
            Ok(picker) => {
                self.upload_picker = Some(picker);
                self.input_mode = InputMode::UploadPicker;
                self.status = "Upload picker".to_string();
            }
            Err(err) => {
                self.status = format!("Failed to open upload picker: {}", err);
            }
        }
    }

    fn start_quit_confirm(&mut self) {
        self.input_mode = InputMode::QuitConfirm;
        self.status = "Confirm quit".to_string();
    }

    fn request_exit(&mut self) {
        self.exit_requested = true;
        if let Some(job) = &self.upload_job {
            job.cancel.store(true, Ordering::SeqCst);
        }
        if let Some(job) = &self.download_job {
            job.cancel.store(true, Ordering::SeqCst);
        }
    }

    fn can_quit(&self) -> bool {
        self.upload_job.is_none() && self.download_job.is_none()
    }

    fn has_active_transfer(&self) -> bool {
        !self.can_quit()
    }

    fn should_exit(&self) -> bool {
        self.exit_requested && self.can_quit()
    }

    fn start_delete_confirm(&mut self) {
        let item = match self.items.get(self.selected) {
            Some(item) => item.clone(),
            None => {
                self.status = "No selection".to_string();
                return;
            }
        };
        if item.is_parent {
            self.status = "Cannot delete parent entry".to_string();
            return;
        }
        if item.id.is_empty() {
            self.status = "Missing file id".to_string();
            return;
        }
        self.pending_delete = Some(item);
        self.input_mode = InputMode::DeleteConfirm;
        self.status = "Confirm delete".to_string();
    }

    fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.items.len();
    }

    fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    fn reload(&mut self, handle: &Handle) -> Result<(), Error> {
        let query = match self.current_folder_id.clone() {
            Some(folder_id) => ListQuery::FilesInFolder { folder_id },
            None => ListQuery::RootNotTrashed,
        };
        let files = handle
            .block_on(files::list::list_files(
                &self.hub,
                &ListFilesConfig {
                    query,
                    order_by: ListSortOrder::default(),
                    max_files: 1000,
                },
            ))
            .map_err(Error::List)?;

        self.items = files
            .into_iter()
            .map(|file| DriveItem {
                id: file.id.clone().unwrap_or_default(),
                name: file.name.clone().unwrap_or_else(|| "<unnamed>".to_string()),
                is_folder: drive_file::is_directory(&file),
                size: file.size,
                is_parent: false,
            })
            .collect();
        self.items.push(DriveItem {
            id: String::new(),
            name: "/..".to_string(),
            is_folder: true,
            size: None,
            is_parent: true,
        });
        self.items.sort_by(|a, b| match (a.is_folder, b.is_folder) {
            _ if a.is_parent && !b.is_parent => std::cmp::Ordering::Less,
            _ if b.is_parent && !a.is_parent => std::cmp::Ordering::Greater,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        self.selected = 0;
        self.status = "Ready".to_string();
        Ok(())
    }

    fn open_selected(&mut self, handle: &Handle) -> Result<(), Error> {
        let item = match self.items.get(self.selected) {
            Some(item) => item.clone(),
            None => {
                self.status = "No selection".to_string();
                return Ok(());
            }
        };
        if !item.is_folder {
            self.status = "Not a folder".to_string();
            return Ok(());
        }
        if item.is_parent {
            return self.go_back(handle);
        }
        if item.id.is_empty() {
            self.status = "Missing folder id".to_string();
            return Ok(());
        }

        let previous = FolderState {
            id: self.current_folder_id.clone(),
            name: self.current_folder_name.clone(),
        };
        self.folder_stack.push(previous);
        self.current_folder_id = Some(item.id);
        self.current_folder_name = item.name;
        self.reload(handle)
    }

    fn delete_item(&mut self, handle: &Handle, item: DriveItem) -> Result<(), Error> {
        let config = files::delete::Config {
            file_id: item.id,
            delete_directories: item.is_folder,
        };
        handle.block_on(files::delete(config)).map_err(Error::Delete)
    }

    fn go_back(&mut self, handle: &Handle) -> Result<(), Error> {
        let previous = match self.folder_stack.pop() {
            Some(folder) => folder,
            None => {
                self.status = "Already at root".to_string();
                return Ok(());
            }
        };
        self.current_folder_id = previous.id;
        self.current_folder_name = previous.name;
        self.reload(handle)
    }

    fn download_selected(
        &mut self,
        handle: &Handle,
        destination: Option<PathBuf>,
    ) -> Result<(), Error> {
        if self.download_job.is_some() {
            self.status = "Download already in progress".to_string();
            return Ok(());
        }
        let item = match self.items.get(self.selected) {
            Some(item) => item.clone(),
            None => {
                self.status = "No selection".to_string();
                return Ok(());
            }
        };
        if item.is_folder {
            self.status = "Select a file to download".to_string();
            return Ok(());
        }
        if item.id.is_empty() {
            self.status = "Missing file id".to_string();
            return Ok(());
        }

        let progress = DownloadProgress::new(item.name.clone());
        let shared_progress = std::sync::Arc::new(std::sync::Mutex::new(progress));
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        let progress_ref = shared_progress.clone();
        let file_id = item.id.clone();
        let destination = destination.clone();
        let handle = handle.clone();
        let cancel_ref = cancel.clone();
        let join_handle = std::thread::spawn(move || {
            let result =
                handle.block_on(download_with_progress(file_id, destination, progress_ref.clone(), cancel_ref));
            if let Ok(mut progress) = progress_ref.lock() {
                progress.done = true;
                if let Err(err) = result {
                    progress.error = Some(err);
                }
            }
        });

        self.download_job = Some(DownloadJob {
            progress: shared_progress,
            handle: Some(join_handle),
            cancel,
        });
        self.status = "Download started".to_string();
        Ok(())
    }

    fn start_upload_job(&mut self, handle: &Handle, path: PathBuf) -> Result<(), Error> {
        if self.upload_job.is_some() {
            self.status = "Upload already in progress".to_string();
            return Ok(());
        }

        let parents = self.current_folder_id.clone().map(|id| vec![id]);
        let progress = UploadProgress::new();
        let shared_progress = std::sync::Arc::new(std::sync::Mutex::new(progress));
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        let progress_ref = shared_progress.clone();
        let handle = handle.clone();
        let cancel_ref = cancel.clone();
        let join_handle = std::thread::spawn(move || {
            let result =
                handle.block_on(upload_with_progress(path, parents, progress_ref.clone(), cancel_ref));
            if let Ok(mut progress) = progress_ref.lock() {
                progress.done = true;
                if let Err(err) = result {
                    progress.error = Some(err);
                }
            }
        });

        self.upload_job = Some(UploadJob {
            progress: shared_progress,
            handle: Some(join_handle),
            cancel,
        });
        self.status = "Upload started".to_string();
        Ok(())
    }

    fn tick(&mut self) {
        if self.last_blink.elapsed() >= Duration::from_millis(500) {
            self.blink_on = !self.blink_on;
            self.last_blink = Instant::now();
        }

        if let Some(job) = &mut self.upload_job {
            let done = job
                .progress
                .lock()
                .map(|progress| progress.done)
                .unwrap_or(false);
            if done {
                let mut refresh_needed = false;
                if let Some(handle) = job.handle.take() {
                    let _ = handle.join();
                }
                if let Ok(progress) = job.progress.lock() {
                    if let Some(error) = progress.error.clone() {
                        self.status = format!("Upload failed: {}", error);
                    } else {
                        self.status = "Upload completed".to_string();
                        refresh_needed = true;
                    }
                }
                self.upload_job = None;
                if refresh_needed {
                    if let Err(err) = self.reload(&Handle::current()) {
                        self.status = format!("Upload completed (refresh failed: {})", err);
                    }
                }
            }
        }

        if let Some(job) = &mut self.download_job {
            let done = job
                .progress
                .lock()
                .map(|progress| progress.done)
                .unwrap_or(false);
            if done {
                if let Some(handle) = job.handle.take() {
                    let _ = handle.join();
                }
                if let Ok(progress) = job.progress.lock() {
                    if let Some(error) = progress.error.clone() {
                        self.status = format!("Download failed: {}", error);
                    } else {
                        self.status = "Download completed".to_string();
                    }
                }
                self.download_job = None;
            }
        }
    }

    fn render_status(&self) -> String {
        if let Some(job) = &self.upload_job {
            if let Ok(progress) = job.progress.lock() {
                if let Some(total_files) = progress.total_files {
                    let current = progress
                        .current_file
                        .clone()
                        .unwrap_or_else(|| "<unknown>".to_string());
                    let file_info = if let Some(total_bytes) = progress.total_bytes {
                        format!(
                            "{} ({}/{})",
                            current,
                            human_bytes(progress.current_bytes as f64),
                            human_bytes(total_bytes as f64)
                        )
                    } else {
                        current
                    };
                    return format!(
                        "Uploading {} [{}/{}]",
                        file_info,
                        progress.done_files,
                        total_files
                    );
                }
                if let Some(total_bytes) = progress.total_bytes {
                    return format!(
                        "Uploading ({}/{})",
                        human_bytes(progress.current_bytes as f64),
                        human_bytes(total_bytes as f64)
                    );
                }
                return "Uploading...".to_string();
            }
        }

        if let Some(job) = &self.download_job {
            if let Ok(progress) = job.progress.lock() {
                let total = progress.total_bytes;
                let current = progress.current_bytes;
                if let Some(total_bytes) = total {
                    return format!(
                        "Downloading {} ({}/{})",
                        progress.file_name,
                        human_bytes(current as f64),
                        human_bytes(total_bytes as f64)
                    );
                }
                return format!("Downloading {} ({})", progress.file_name, human_bytes(current as f64));
            }
        }
        self.status.clone()
    }
}

impl UploadPicker {
    fn from_dir(path: PathBuf) -> Result<Self, io::Error> {
        let entries = list_local_entries(&path)?;
        Ok(Self {
            current_dir: path,
            entries,
            selected: 0,
            selected_path: None,
        })
    }

    fn reload(&mut self) -> Result<(), io::Error> {
        self.entries = list_local_entries(&self.current_dir)?;
        self.selected = 0;
        Ok(())
    }
}

fn list_local_entries(path: &PathBuf) -> Result<Vec<LocalEntry>, io::Error> {
    let mut entries: Vec<LocalEntry> = vec![];
    let parent_path = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| path.clone());
    entries.push(LocalEntry {
        name: "/..".to_string(),
        path: parent_path,
        is_dir: true,
        is_parent: true,
    });

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();
        let is_dir = entry_path.is_dir();
        if entry_path.is_file() || is_dir {
            entries.push(LocalEntry {
                name,
                path: entry_path,
                is_dir,
                is_parent: false,
            });
        }
    }

    entries.sort_by(|a, b| {
        if a.is_parent && !b.is_parent {
            return std::cmp::Ordering::Less;
        }
        if b.is_parent && !a.is_parent {
            return std::cmp::Ordering::Greater;
        }
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Hub(hub_helper::Error),
    List(files::list::Error),
    Download(files::download::Error),
    Delete(files::delete::Error),
    Upload(files::upload::Error),
    Join(tokio::task::JoinError),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{}", err),
            Error::Hub(err) => write!(f, "{}", err),
            Error::List(err) => write!(f, "{}", err),
            Error::Download(err) => write!(f, "{}", err),
            Error::Delete(err) => write!(f, "{}", err),
            Error::Upload(err) => write!(f, "{}", err),
            Error::Join(err) => write!(f, "{}", err),
        }
    }
}

struct UploadJob {
    progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
    handle: Option<std::thread::JoinHandle<()>>,
    cancel: std::sync::Arc<AtomicBool>,
}

struct UploadProgress {
    current_file: Option<String>,
    current_bytes: u64,
    total_bytes: Option<u64>,
    done_files: u64,
    total_files: Option<u64>,
    done: bool,
    error: Option<String>,
}

impl UploadProgress {
    fn new() -> Self {
        Self {
            current_file: None,
            current_bytes: 0,
            total_bytes: None,
            done_files: 0,
            total_files: None,
            done: false,
            error: None,
        }
    }
}

struct DownloadJob {
    progress: std::sync::Arc<std::sync::Mutex<DownloadProgress>>,
    handle: Option<std::thread::JoinHandle<()>>,
    cancel: std::sync::Arc<AtomicBool>,
}

struct DownloadProgress {
    file_name: String,
    current_bytes: u64,
    total_bytes: Option<u64>,
    done: bool,
    error: Option<String>,
}

impl DownloadProgress {
    fn new(file_name: String) -> Self {
        Self {
            file_name,
            current_bytes: 0,
            total_bytes: None,
            done: false,
            error: None,
        }
    }
}

struct ProgressReader<R> {
    inner: R,
    progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
    position: u64,
}

impl<R> ProgressReader<R> {
    fn new(
        inner: R,
        progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
        cancel: std::sync::Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner,
            progress,
            cancel,
            position: 0,
        }
    }
}

impl<R: std::io::Read + std::io::Seek> std::io::Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Cancelled"));
        }
        let count = self.inner.read(buf)?;
        if count > 0 {
            self.position = self.position.saturating_add(count as u64);
            if let Ok(mut progress) = self.progress.lock() {
                progress.current_bytes = self.position;
            }
        }
        Ok(count)
    }
}

impl<R: std::io::Read + std::io::Seek> std::io::Seek for ProgressReader<R> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Cancelled"));
        }
        let new_pos = self.inner.seek(pos)?;
        self.position = new_pos;
        if let Ok(mut progress) = self.progress.lock() {
            progress.current_bytes = new_pos;
        }
        Ok(new_pos)
    }
}

async fn download_with_progress(
    file_id: String,
    destination: Option<PathBuf>,
    progress: std::sync::Arc<std::sync::Mutex<DownloadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<(), String> {
    let hub = hub_helper::get_hub().await.map_err(|err| err.to_string())?;
    let file = files::info::get_file(&hub, &file_id)
        .await
        .map_err(|err| err.to_string())?;

    if drive_file::is_directory(&file) {
        return Err("Selected item is a directory".to_string());
    }
    if drive_file::is_shortcut(&file) {
        return Err("Shortcuts are not supported in TUI download".to_string());
    }

    let file_name = file
        .name
        .clone()
        .ok_or_else(|| "File does not have a name".to_string())?;
    if let Ok(mut progress) = progress.lock() {
        progress.file_name = file_name.clone();
        progress.total_bytes = file.size.and_then(|size| u64::try_from(size).ok());
    }

    let root_path = match destination {
        Some(path) => {
            if !path.exists() {
                return Err(format!("Destination path '{}' does not exist", path.display()));
            }
            if !path.is_dir() {
                return Err(format!(
                    "Destination path '{}' is not a directory",
                    path.display()
                ));
            }
            path.canonicalize()
                .map_err(|err| format!("Failed to canonicalize destination: {}", err))?
        }
        None => std::path::PathBuf::from(".")
            .canonicalize()
            .map_err(|err| format!("Failed to canonicalize destination: {}", err))?,
    };

    let file_path = root_path.join(&file_name);
    if file_path.exists() {
        return Err(format!(
            "File '{}' already exists, delete it or use a different destination",
            file_path.display()
        ));
    }

    let body = files::download::download_file(&hub, &file_id)
        .await
        .map_err(|err| err.to_string())?;

    save_body_to_file_with_progress(
        body,
        &file_path,
        file.md5_checksum.clone(),
        progress,
        cancel,
    )
        .await
}

async fn save_body_to_file_with_progress(
    mut body: hyper::Body,
    file_path: &PathBuf,
    expected_md5: Option<String>,
    progress: std::sync::Arc<std::sync::Mutex<DownloadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<(), String> {
    let tmp_file_path = file_path.with_extension("incomplete");
    let file = std::fs::File::create(&tmp_file_path).map_err(|err| err.to_string())?;
    let mut writer = Md5Writer::new(file);
    let mut total_written: u64 = 0;

    while let Some(chunk_result) = body.next().await {
        if cancel.load(Ordering::SeqCst) {
            let _ = std::fs::remove_file(&tmp_file_path);
            return Err("Cancelled".to_string());
        }
        let chunk = chunk_result.map_err(|err| err.to_string())?;
        writer.write_all(&chunk).map_err(|err| err.to_string())?;
        total_written = total_written.saturating_add(chunk.len() as u64);
        if let Ok(mut progress) = progress.lock() {
            progress.current_bytes = total_written;
        }
    }

    let actual_md5 = writer.md5();
    if let Some(expected) = expected_md5 {
        if expected != actual_md5 {
            return Err(format!(
                "MD5 mismatch, expected: {}, actual: {}",
                expected, actual_md5
            ));
        }
    }

    std::fs::rename(&tmp_file_path, &file_path).map_err(|err| err.to_string())
}

async fn upload_with_progress(
    path: PathBuf,
    parents: Option<Vec<String>>,
    progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<(), String> {
    let hub = hub_helper::get_hub().await.map_err(|err| err.to_string())?;
    let delegate_config = UploadDelegateConfig {
        chunk_size: ChunkSize::default(),
        backoff_config: BackoffConfig {
            max_retries: 100000,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(60),
        },
        print_chunk_errors: false,
        print_chunk_info: false,
    };

    if path.is_dir() {
        upload_directory_with_progress(
            &hub,
            path,
            parents,
            delegate_config,
            progress,
            cancel,
        )
        .await
    } else {
        upload_single_file_with_progress(
            &hub,
            path,
            parents,
            delegate_config,
            progress,
            cancel,
        )
        .await
    }
}

async fn upload_single_file_with_progress(
    hub: &Hub,
    path: PathBuf,
    parents: Option<Vec<String>>,
    delegate_config: UploadDelegateConfig,
    progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        return Err("Cancelled".to_string());
    }
    let file = std::fs::File::open(&path).map_err(|err| err.to_string())?;
    let file_info = file_info::FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: path.clone(),
            mime_type: None,
            parents,
        },
    )
    .map_err(|err| err.to_string())?;

    if let Ok(mut progress) = progress.lock() {
        progress.current_file = Some(file_info.name.clone());
        progress.total_bytes = Some(file_info.size);
        progress.current_bytes = 0;
        progress.done_files = 0;
        progress.total_files = Some(1);
    }

    let reader = ProgressReader::new(file, progress.clone(), cancel);
    upload::upload_file(hub, reader, None, file_info, delegate_config)
        .await
        .map_err(|err| err.to_string())?;

    if let Ok(mut progress) = progress.lock() {
        progress.done_files = 1;
    }

    Ok(())
}

async fn upload_directory_with_progress(
    hub: &Hub,
    path: PathBuf,
    parents: Option<Vec<String>>,
    delegate_config: UploadDelegateConfig,
    progress: std::sync::Arc<std::sync::Mutex<UploadProgress>>,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        return Err("Cancelled".to_string());
    }
    let mut ids = IdGen::new(hub, &delegate_config);
    let tree = file_tree::FileTree::from_path(&path, &mut ids)
        .await
        .map_err(|err| err.to_string())?;

    let tree_info = tree.info();
    if let Ok(mut progress) = progress.lock() {
        progress.total_files = Some(tree_info.file_count as u64);
        progress.done_files = 0;
    }

    for folder in tree.folders() {
        if cancel.load(Ordering::SeqCst) {
            return Err("Cancelled".to_string());
        }
        let folder_parents = folder
            .parent
            .as_ref()
            .map(|p| vec![p.drive_id.clone()])
            .or_else(|| parents.clone());

        let drive_folder = mkdir::create_directory(
            hub,
            &mkdir::Config {
                id: Some(folder.drive_id.clone()),
                name: folder.name.clone(),
                parents: folder_parents,
                print_only_id: false,
            },
            delegate_config.clone(),
        )
        .await
        .map_err(|err| err.to_string())?;

        let folder_id = drive_folder.id.ok_or("Folder created on drive has no id")?;
        let file_parents = Some(vec![folder_id.clone()]);

        for file in folder.files() {
            if cancel.load(Ordering::SeqCst) {
                return Err("Cancelled".to_string());
            }
            if let Ok(mut progress) = progress.lock() {
                progress.current_file = Some(file.relative_path().display().to_string());
                progress.total_bytes = Some(file.size);
                progress.current_bytes = 0;
            }

            let os_file = std::fs::File::open(&file.path).map_err(|err| err.to_string())?;
            let reader = ProgressReader::new(os_file, progress.clone(), cancel.clone());
            let file_info = file.info(file_parents.clone());

            upload::upload_file(
                hub,
                reader,
                Some(file.drive_id.clone()),
                file_info,
                delegate_config.clone(),
            )
            .await
            .map_err(|err| err.to_string())?;

            if let Ok(mut progress) = progress.lock() {
                progress.done_files = progress.done_files.saturating_add(1);
            }
        }
    }

    Ok(())
}
