use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde_derive::{Deserialize, Serialize};
use std::{error::Error, fs, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Serialize, Deserialize, Clone)]
struct App {
    input: String,
    todos: Vec<String>,
    index: usize,
    show_popup: bool,
}

impl App {
    fn default() -> App {
        App {
            input: String::new(),
            todos: Vec::new(),
            index: 0,
            show_popup: false,
        }
    }
    fn next(&mut self) {
        self.index = (self.index + 1) % self.todos.len();
    }
    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.todos.len() - 1;
        }
    }
    fn chain_hook(&mut self) {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            reset_terminal().unwrap();
            original_hook(panic);
        }));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn reset_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.chain_hook();

    let todo_path: &String = &format!("/home/{}/.config/todos.json", env!("USER"));

    if let Ok(todos_json) = fs::read_to_string(todo_path) {
        let todos: Vec<String> = serde_json::from_str(&todos_json)?;
        app.todos = todos;
    } else {
        let todos_json = serde_json::to_vec(&app.todos)?;
        fs::write(todo_path, todos_json)?;
    }

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let time = chrono::Local::now().format("%B %d %I:%M %p");

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    if !app.input.is_empty() {
                        app.todos.push(format!(
                            "{} [{}]",
                            app.input.drain(..).collect::<String>(),
                            time
                        ));
                    } else {
                        app.show_popup = !app.show_popup
                    }
                }
                KeyCode::Up => {
                    if !app.todos.is_empty() {
                        app.previous();
                    }
                }
                KeyCode::Down => {
                    if !app.todos.is_empty() {
                        app.next();
                    }
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Tab => {
                    if !app.todos.is_empty() {
                        if app.index < app.todos.len() {
                            app.todos.remove(app.index);
                        } else {
                            app.index = app.todos.len() - 1;
                        }
                    }
                }
                KeyCode::Esc => {
                    let json = serde_json::to_vec(&app.todos)?;
                    fs::write(todo_path, json)?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(f.size());

        let (msg, style) = (
            vec![
                Span::raw("Press "),
                Span::styled(
                    "Up/Down key",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
                Span::raw(" to navigate, "),
                Span::styled(
                    "Tab",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
                Span::raw(" to remove TODO, "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
                Span::raw(" to exit. "),
            ],
            Style::default().add_modifier(Modifier::BOLD),
        );
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, chunks[0]);

        let input = Paragraph::new(app.input.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Add a TODO"));
        f.render_widget(input, chunks[1]);
        f.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1);

        let todos: Vec<ListItem> = app
            .todos
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i + 1, m)))];
                ListItem::new(content)
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(app.index));

        let todos = List::new(todos)
            .block(Block::default().borders(Borders::ALL).title("Todo(s)"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .highlight_symbol("> ");
        f.render_stateful_widget(todos, chunks[2], &mut state);

        if app.index < app.todos.len() {
            if app.show_popup {
                let block = Paragraph::new(format!("{}", app.todos[app.index]))
                    .block(Block::default().borders(Borders::ALL).title("More info"));
                let area = centered_rect(60, 20, f.size());
                f.render_widget(Clear, area); //this clears out the background
                f.render_widget(block, area);
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
