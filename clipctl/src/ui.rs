use std::io::Stdout;

use anyhow::Result;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;

use crate::ipc::{EntrySummary, Request, RequestKind, Response};
use crate::paste::{PasteEngine, PasteMethod};

#[derive(Debug)]
pub enum UiEvent {
    Input(crossterm::event::Event),
}

pub struct HandleOutcome {
    pub should_exit: bool,
    pub request: Option<Request>,
}

pub struct TerminalUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    entries: Vec<EntrySummary>,
    selected: usize,
    filter: String,
    paste: PasteEngine,
    list_state: ListState,
    mode: UiMode,
    input_buffer: String,
}

#[derive(Debug, Clone, PartialEq)]
enum UiMode {
    Normal,
    Search,
    AddTag,
    RemoveTag,
    Export,
    Import,
    Help,
}

impl TerminalUi {
    pub fn new() -> Result<Self> {
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Self {
            terminal,
            entries: Vec::new(),
            selected: 0,
            filter: String::new(),
            paste: PasteEngine::new(PasteMethod::SendInput),
            list_state,
            mode: UiMode::Normal,
            input_buffer: String::new(),
        })
    }

    pub fn draw(&mut self) -> Result<()> {
        self.list_state.select(if self.entries.is_empty() {
            None
        } else {
            Some(self.selected)
        });

        let is_help_mode = self.mode == UiMode::Help;
        let list_state = &mut self.list_state;
        let entries = &self.entries;
        let selected = self.selected;
        let mode = &self.mode;
        let filter = &self.filter;
        let input_buffer = &self.input_buffer;

        self.terminal.draw(|frame| {
            let size = frame.size();
            
            // If in help mode, show help screen
            if is_help_mode {
                let help_text = vec![
                    "rusty-clipboard - Keybindings",
                    "",
                    "Navigation:",
                    "  j/↓         Move down",
                    "  k/↑         Move up",
                    "  g           Go to top",
                    "  G           Go to bottom",
                    "",
                    "Actions:",
                    "  Enter/l     Paste selected entry",
                    "  /           Start search",
                    "  t           Add tag to entry",
                    "  T           Remove tag from entry",
                    "  e           Export history to JSON",
                    "  i           Import history from JSON",
                    "",
                    "General:",
                    "  ?           Show this help",
                    "  q/Esc       Quit",
                    "",
                    "Press any key to close help...",
                ];
                
                let help = Paragraph::new(help_text.join("\n"))
                    .block(
                        Block::default()
                            .title("Help")
                            .borders(Borders::ALL)
                            .title_alignment(Alignment::Center),
                    )
                    .alignment(Alignment::Left)
                    .wrap(ratatui::widgets::Wrap { trim: false });
                
                frame.render_widget(help, size);
                return;
            }
            
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .split(size);
            let main = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(layout[0]);

            // Format history items with kind and tags
            let history_items: Vec<_> = entries
                .iter()
                .map(|entry| {
                    let kind_icon = match entry.kind.as_str() {
                        "text" => "📝",
                        "url" => "🔗",
                        "image" => "🖼️",
                        "rtf" => "📄",
                        _ => "❓",
                    };
                    let tags_str = if entry.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", entry.tags.join(", "))
                    };
                    ListItem::new(format!("{} {}{}", kind_icon, entry.preview, tags_str))
                })
                .collect();

            let list = List::new(history_items)
                .block(
                    Block::default()
                        .title("History (? for help)")
                        .borders(Borders::ALL)
                        .title_alignment(Alignment::Center),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            // Enhanced preview with metadata
            let preview_text = entries
                .get(selected)
                .map(|e| {
                    let mut text = String::new();
                    text.push_str(&format!("Type: {}\n", e.kind));
                    if let Some(ref proc) = e.source_process {
                        text.push_str(&format!("Source: {}\n", proc));
                    }
                    if !e.tags.is_empty() {
                        text.push_str(&format!("Tags: {}\n", e.tags.join(", ")));
                    }
                    text.push_str(&format!("Time: {}\n\n", e.created_at));
                    text.push_str(&e.preview);
                    text
                })
                .unwrap_or_else(|| "<no selection>".to_string());

            let preview = Paragraph::new(preview_text)
                .block(Block::default().title("Preview").borders(Borders::ALL))
                .alignment(Alignment::Left)
                .wrap(ratatui::widgets::Wrap { trim: true });

            // Mode-aware command bar
            let (command_prompt, command_text) = match mode {
                UiMode::Normal => ("Search: /", filter.as_str()),
                UiMode::Search => ("Search: /", input_buffer.as_str()),
                UiMode::AddTag => ("Add tag: ", input_buffer.as_str()),
                UiMode::RemoveTag => ("Remove tag: ", input_buffer.as_str()),
                UiMode::Export => ("Export to: ", input_buffer.as_str()),
                UiMode::Import => ("Import from: ", input_buffer.as_str()),
                UiMode::Help => ("", ""),
            };

            let command_bar = Paragraph::new(format!("{}{}", command_prompt, command_text))
                .block(Block::default().borders(Borders::ALL).title("Command"))
                .alignment(Alignment::Left);

            frame.render_stateful_widget(list, main[0], list_state);
            frame.render_widget(preview, main[1]);
            frame.render_widget(command_bar, layout[1]);
        })?;
        Ok(())
    }

    pub fn handle_event(&mut self, event: UiEvent) -> Result<HandleOutcome> {
        let mut request = None;
        let mut should_exit = false;

        let UiEvent::Input(ev) = event;
        match ev {
            crossterm::event::Event::Key(key) => {
                use crossterm::event::{KeyCode, KeyEventKind};
                if key.kind == KeyEventKind::Press {
                    // Handle help mode separately
                    if self.mode == UiMode::Help {
                        self.mode = UiMode::Normal;
                        return Ok(HandleOutcome { should_exit, request });
                    }
                    
                    // Handle input modes (AddTag, RemoveTag, Export, Import, Search)
                    if self.mode != UiMode::Normal {
                        match key.code {
                            KeyCode::Esc => {
                                self.mode = UiMode::Normal;
                                self.input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                request = self.handle_input_mode_submit()?;
                                self.mode = UiMode::Normal;
                                self.input_buffer.clear();
                            }
                            KeyCode::Backspace => {
                                self.input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                self.input_buffer.push(c);
                                // For search mode, update results in real-time
                                if self.mode == UiMode::Search {
                                    self.filter = self.input_buffer.clone();
                                    request = Some(Request {
                                        kind: RequestKind::Search {
                                            query: self.filter.clone(),
                                        },
                                    });
                                }
                            }
                            _ => {}
                        }
                        return Ok(HandleOutcome { should_exit, request });
                    }
                    
                    // Normal mode keybindings
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => should_exit = true,
                        KeyCode::Char('?') => {
                            self.mode = UiMode::Help;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if !self.entries.is_empty() {
                                self.selected = (self.selected + 1).min(self.entries.len() - 1);
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if self.selected > 0 {
                                self.selected -= 1;
                            }
                        }
                        KeyCode::Char('g') => {
                            self.selected = 0;
                        }
                        KeyCode::Char('G') => {
                            if !self.entries.is_empty() {
                                self.selected = self.entries.len() - 1;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char('l') => {
                            if let Some(entry) = self.entries.get(self.selected) {
                                self.paste.paste(&entry.preview)?;
                                request = Some(Request {
                                    kind: RequestKind::Paste { id: entry.id },
                                });
                                should_exit = true;
                            }
                        }
                        KeyCode::Char('/') => {
                            self.mode = UiMode::Search;
                            self.input_buffer = self.filter.clone();
                        }
                        KeyCode::Char('t') => {
                            self.mode = UiMode::AddTag;
                            self.input_buffer.clear();
                        }
                        KeyCode::Char('T') => {
                            self.mode = UiMode::RemoveTag;
                            self.input_buffer.clear();
                        }
                        KeyCode::Char('e') => {
                            self.mode = UiMode::Export;
                            self.input_buffer = "clipboard_export.json".to_string();
                        }
                        KeyCode::Char('i') => {
                            self.mode = UiMode::Import;
                            self.input_buffer = "clipboard_export.json".to_string();
                        }
                        _ => {}
                    }
                }
            }
            crossterm::event::Event::Resize(_, _) => {}
            _ => {}
        }

        Ok(HandleOutcome { should_exit, request })
    }
    
    fn handle_input_mode_submit(&self) -> Result<Option<Request>> {
        if self.input_buffer.is_empty() {
            return Ok(None);
        }
        
        let current_entry = self.entries.get(self.selected);
        
        match self.mode {
            UiMode::Search => {
                Ok(Some(Request {
                    kind: RequestKind::Search {
                        query: self.input_buffer.clone(),
                    },
                }))
            }
            UiMode::AddTag => {
                if let Some(entry) = current_entry {
                    Ok(Some(Request {
                        kind: RequestKind::AddTag {
                            id: entry.id,
                            tag: self.input_buffer.clone(),
                        },
                    }))
                } else {
                    Ok(None)
                }
            }
            UiMode::RemoveTag => {
                if let Some(entry) = current_entry {
                    Ok(Some(Request {
                        kind: RequestKind::RemoveTag {
                            id: entry.id,
                            tag: self.input_buffer.clone(),
                        },
                    }))
                } else {
                    Ok(None)
                }
            }
            UiMode::Export => {
                Ok(Some(Request {
                    kind: RequestKind::Export {
                        path: self.input_buffer.clone(),
                    },
                }))
            }
            UiMode::Import => {
                Ok(Some(Request {
                    kind: RequestKind::Import {
                        path: self.input_buffer.clone(),
                    },
                }))
            }
            _ => Ok(None),
        }
    }

    pub fn ingest_response(&mut self, response: Response) -> Result<()> {
        if response.entries.is_empty() {
            self.selected = 0;
        } else if self.selected >= response.entries.len() {
            self.selected = response.entries.len() - 1;
        }
        self.entries = response.entries;
        Ok(())
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        let _ = self.terminal.show_cursor();
        let _ = crossterm::execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

