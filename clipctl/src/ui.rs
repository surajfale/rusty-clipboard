use std::io::Stdout;

use anyhow::Result;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;

use crate::ipc::{EntrySummary, Request, RequestKind, Response};
use crate::paste::{PasteEngine, PasteMethod};
use crate::syntax::{detect_code_language, highlight_code, render_formatted_text};
use crate::theme::Theme;

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
    theme: Theme,
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
            theme: Theme::nord(), // Default to Nord theme, can be made configurable
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
                let theme = &self.theme;
                
                let help_lines = vec![
                    Line::from(vec![
                        Span::styled(
                            "rusty-clipboard",
                            Style::default()
                                .fg(theme.border_focused)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                        Span::raw(" - Keybindings"),
                    ]),
                    Line::raw(""),
                    Line::styled("Navigation:", theme.style_help_section()),
                    Line::from(vec![
                        Span::styled("  j", theme.style_help_key()),
                        Span::raw("/"),
                        Span::styled("↓", theme.style_help_key()),
                        Span::styled("         Move down", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  k", theme.style_help_key()),
                        Span::raw("/"),
                        Span::styled("↑", theme.style_help_key()),
                        Span::styled("         Move up", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  g", theme.style_help_key()),
                        Span::styled("           Go to top", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  G", theme.style_help_key()),
                        Span::styled("           Go to bottom", theme.style_help_desc()),
                    ]),
                    Line::raw(""),
                    Line::styled("Actions:", theme.style_help_section()),
                    Line::from(vec![
                        Span::styled("  Enter", theme.style_help_key()),
                        Span::raw("/"),
                        Span::styled("l", theme.style_help_key()),
                        Span::styled("     Paste selected entry", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  /", theme.style_help_key()),
                        Span::styled("           Start search", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  t", theme.style_help_key()),
                        Span::styled("           Add tag to entry", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  T", theme.style_help_key()),
                        Span::styled("           Remove tag from entry", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  e", theme.style_help_key()),
                        Span::styled("           Export history to JSON", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  i", theme.style_help_key()),
                        Span::styled("           Import history from JSON", theme.style_help_desc()),
                    ]),
                    Line::raw(""),
                    Line::styled("General:", theme.style_help_section()),
                    Line::from(vec![
                        Span::styled("  ?", theme.style_help_key()),
                        Span::styled("           Show this help", theme.style_help_desc()),
                    ]),
                    Line::from(vec![
                        Span::styled("  q", theme.style_help_key()),
                        Span::raw("/"),
                        Span::styled("Esc", theme.style_help_key()),
                        Span::styled("       Quit", theme.style_help_desc()),
                    ]),
                    Line::raw(""),
                    Line::styled(
                        "Press any key to close help...",
                        Style::default().fg(theme.metadata_label).add_modifier(Modifier::ITALIC),
                    ),
                ];
                
                let help = Paragraph::new(help_lines)
                    .block(
                        Block::default()
                            .title(Span::styled(" Help ", theme.style_title()))
                            .borders(Borders::ALL)
                            .border_style(theme.style_border_focused())
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
            let theme = &self.theme;
            let history_items: Vec<_> = entries
                .iter()
                .map(|entry| {
                    let (kind_icon, icon_color) = match entry.kind.as_str() {
                        "text" => ("📝", theme.text_icon),
                        "url" => ("🔗", theme.url_icon),
                        "image" => ("🖼️", theme.image_icon),
                        "rtf" => ("📄", theme.rtf_icon),
                        _ => ("❓", theme.metadata_label),
                    };
                    
                    let mut spans = vec![
                        Span::styled(
                            format!("{} ", kind_icon),
                            Style::default().fg(icon_color),
                        ),
                    ];
                    
                    // Truncate preview if too long
                    let preview_text = if entry.preview.len() > 80 {
                        format!("{}...", &entry.preview[..77])
                    } else {
                        entry.preview.clone()
                    };
                    
                    spans.push(Span::styled(
                        preview_text,
                        theme.style_list_item(),
                    ));
                    
                    // Add tags with styling
                    if !entry.tags.is_empty() {
                        spans.push(Span::raw("  "));
                        for (i, tag) in entry.tags.iter().enumerate() {
                            if i > 0 {
                                spans.push(Span::raw(" "));
                            }
                            spans.push(Span::styled(
                                format!(" {} ", tag),
                                theme.style_tag(),
                            ));
                        }
                    }
                    
                    ListItem::new(Line::from(spans))
                })
                .collect();

            let list = List::new(history_items)
                .block(
                    Block::default()
                        .title(Span::styled(" History (? for help) ", theme.style_title()))
                        .borders(Borders::ALL)
                        .border_style(theme.style_border())
                        .title_alignment(Alignment::Center),
                )
                .highlight_style(theme.style_list_selected())
                .highlight_symbol("▶ ");

            // Enhanced preview with metadata and syntax highlighting
            let preview_content = entries
                .get(selected)
                .map(|e| {
                    let theme = &self.theme;
                    let mut lines = Vec::new();
                    
                    // Metadata header
                    lines.push(Line::from(vec![
                        Span::styled("Type: ", theme.style_metadata_label()),
                        Span::styled(&e.kind, theme.style_metadata_value()),
                    ]));
                    
                    if let Some(ref proc) = e.source_process {
                        lines.push(Line::from(vec![
                            Span::styled("Source: ", theme.style_metadata_label()),
                            Span::styled(proc, theme.style_metadata_value()),
                        ]));
                    }
                    
                    if !e.tags.is_empty() {
                        let mut tag_spans = vec![
                            Span::styled("Tags: ", theme.style_metadata_label()),
                        ];
                        for (i, tag) in e.tags.iter().enumerate() {
                            if i > 0 {
                                tag_spans.push(Span::raw(", "));
                            }
                            tag_spans.push(Span::styled(tag, theme.style_tag()));
                        }
                        lines.push(Line::from(tag_spans));
                    }
                    
                    lines.push(Line::from(vec![
                        Span::styled("Time: ", theme.style_metadata_label()),
                        Span::styled(&e.created_at, theme.style_metadata_value()),
                    ]));
                    
                    lines.push(Line::from(Span::styled(
                        "─".repeat(40),
                        Style::default().fg(theme.border),
                    )));
                    
                    // Content with syntax highlighting or formatting
                    if let Some(lang) = detect_code_language(&e.preview) {
                        // Syntax highlight detected code
                        let highlighted = highlight_code(&e.preview, Some(lang));
                        lines.extend(highlighted.lines);
                    } else if e.preview.contains("# ") || e.preview.contains("## ") {
                        // Render as formatted markdown-like text
                        let formatted = render_formatted_text(&e.preview);
                        lines.extend(formatted.lines);
                    } else {
                        // Regular text with basic styling
                        for line in e.preview.lines().take(50) {
                            lines.push(Line::from(Span::styled(
                                line.to_string(),
                                theme.style_list_item(),
                            )));
                        }
                    }
                    
                    Text::from(lines)
                })
                .unwrap_or_else(|| {
                    Text::from(Line::from(Span::styled(
                        "<no selection>",
                        Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                    )))
                });

            let preview = Paragraph::new(preview_content)
                .block(
                    Block::default()
                        .title(Span::styled(" Preview ", theme.style_title()))
                        .borders(Borders::ALL)
                        .border_style(theme.style_border()),
                )
                .alignment(Alignment::Left)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .scroll((0, 0));

            // Mode-aware command bar with rich styling
            let command_content = match mode {
                UiMode::Normal => {
                    let mut spans = vec![
                        Span::styled("Search: ", theme.style_command_prompt()),
                        Span::styled(filter.as_str(), theme.style_command_input()),
                    ];
                    if filter.is_empty() {
                        spans.push(Span::styled(
                            " (press / to search)",
                            Style::default().fg(theme.metadata_label).add_modifier(Modifier::ITALIC),
                        ));
                    }
                    Line::from(spans)
                }
                UiMode::Search => Line::from(vec![
                    Span::styled("🔍 Search: ", theme.style_command_prompt()),
                    Span::styled(input_buffer.as_str(), theme.style_command_input()),
                    Span::styled("█", Style::default().fg(theme.list_selected_fg)), // Cursor
                ]),
                UiMode::AddTag => Line::from(vec![
                    Span::styled("🏷️  Add tag: ", theme.style_command_prompt()),
                    Span::styled(input_buffer.as_str(), theme.style_command_input()),
                    Span::styled("█", Style::default().fg(theme.list_selected_fg)),
                ]),
                UiMode::RemoveTag => Line::from(vec![
                    Span::styled("🗑️  Remove tag: ", theme.style_command_prompt()),
                    Span::styled(input_buffer.as_str(), theme.style_command_input()),
                    Span::styled("█", Style::default().fg(theme.list_selected_fg)),
                ]),
                UiMode::Export => Line::from(vec![
                    Span::styled("💾 Export to: ", theme.style_command_prompt()),
                    Span::styled(input_buffer.as_str(), theme.style_command_input()),
                    Span::styled("█", Style::default().fg(theme.list_selected_fg)),
                ]),
                UiMode::Import => Line::from(vec![
                    Span::styled("📥 Import from: ", theme.style_command_prompt()),
                    Span::styled(input_buffer.as_str(), theme.style_command_input()),
                    Span::styled("█", Style::default().fg(theme.list_selected_fg)),
                ]),
                UiMode::Help => Line::from(""),
            };

            let command_bar = Paragraph::new(command_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if *mode != UiMode::Normal {
                            theme.style_border_focused()
                        } else {
                            theme.style_border()
                        })
                        .title(Span::styled(" Command ", theme.style_title())),
                )
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

