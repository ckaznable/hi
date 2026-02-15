use std::path::PathBuf;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use tokio::sync::mpsc;

enum SessionCmd {
    Send(String),
    Reset,
}

enum SessionReply {
    StreamChunk(String),
    StreamDone,
    Error(String),
    ResetDone,
}

struct App {
    messages: Vec<(String, String)>,
    input: String,
    waiting: bool,
    should_quit: bool,
    streaming_buffer: String,
}

fn render(frame: &mut Frame, app: &App) {
    let chunks =
        Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(frame.area());

    let mut lines: Vec<Line> = Vec::new();
    for (role, content) in &app.messages {
        let prefix = match role.as_str() {
            "user" => "[You] ",
            "assistant" => "[AI] ",
            _ => "[System] ",
        };
        let style = match role.as_str() {
            "user" => Style::default().fg(Color::Cyan),
            "assistant" => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::DarkGray),
        };
        for line in content.lines() {
            lines.push(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::raw(line),
            ]));
        }
        lines.push(Line::from(""));
    }

    if !app.streaming_buffer.is_empty() {
        let prefix = "[AI] ";
        let style = Style::default().fg(Color::Green);
        for line in app.streaming_buffer.lines() {
            lines.push(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::raw(line.to_string()),
            ]));
        }
    }

    if app.waiting && app.streaming_buffer.is_empty() {
        lines.push(Line::from(Span::styled(
            "Thinking...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    let content_height = lines.len() as u16;
    let visible_height = chunks[0].height.saturating_sub(2);
    let max_scroll = content_height.saturating_sub(visible_height);

    let messages_widget = Paragraph::new(lines)
        .block(Block::bordered().title(" hi "))
        .wrap(Wrap { trim: false })
        .scroll((max_scroll, 0));

    frame.render_widget(messages_widget, chunks[0]);

    let input_title = if app.waiting { " Wait... " } else { " > " };
    let input_widget = Paragraph::new(app.input.as_str())
        .block(Block::bordered().title(input_title));

    frame.render_widget(input_widget, chunks[1]);

    if !app.waiting {
        frame.set_cursor_position(Position::new(
            chunks[1].x + app.input.len() as u16 + 1,
            chunks[1].y + 1,
        ));
    }
}

pub async fn run_tui(config_path: Option<PathBuf>) -> Result<()> {
    let config = match config_path {
        Some(ref p) => shared::config::ModelConfig::load_from_path(p)?,
        None => shared::config::ModelConfig::load()?,
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, config).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: shared::config::ModelConfig,
) -> Result<()> {
    let mut session = hi_core::session::ChatSession::new(config)?;

    let initial_messages: Vec<(String, String)> = session
        .history()
        .messages()
        .iter()
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect();

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCmd>();
    let (reply_tx, mut reply_rx) = mpsc::unbounded_channel::<SessionReply>();

    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                SessionCmd::Send(text) => {
                    let (stream_tx, mut stream_rx) = mpsc::channel::<String>(hi_core::provider::STREAM_CHANNEL_CAPACITY);
                    let forwarder_tx = reply_tx.clone();
                    tokio::spawn(async move {
                        while let Some(chunk) = stream_rx.recv().await {
                            let _ = forwarder_tx.send(SessionReply::StreamChunk(chunk));
                        }
                    });
                    match session.send_message_streaming(&text, stream_tx).await {
                        Ok(_) => {
                            let _ = reply_tx.send(SessionReply::StreamDone);
                        }
                        Err(e) => {
                            let _ = reply_tx.send(SessionReply::Error(format!("{e}")));
                        }
                    }
                }
                SessionCmd::Reset => {
                    let _ = session.reset();
                    let _ = reply_tx.send(SessionReply::ResetDone);
                }
            }
        }
    });

    let mut app = App {
        messages: initial_messages,
        input: String::new(),
        waiting: false,
        should_quit: false,
        streaming_buffer: String::new(),
    };

    loop {
        terminal.draw(|f| render(f, &app))?;

        while let Ok(reply) = reply_rx.try_recv() {
            match reply {
                SessionReply::StreamChunk(chunk) => {
                    app.streaming_buffer.push_str(&chunk);
                }
                SessionReply::StreamDone => {
                    let finished = std::mem::take(&mut app.streaming_buffer);
                    if !finished.is_empty() {
                        app.messages.push(("assistant".to_string(), finished));
                    }
                    app.waiting = false;
                }
                SessionReply::Error(e) => {
                    app.streaming_buffer.clear();
                    app.messages
                        .push(("system".to_string(), format!("Error: {e}")));
                    app.waiting = false;
                }
                SessionReply::ResetDone => {
                    app.messages.clear();
                    app.streaming_buffer.clear();
                    app.waiting = false;
                }
            }
        }

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    app.should_quit = true;
                    continue;
                }

                if app.waiting {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Enter => {
                        let text = app.input.drain(..).collect::<String>();
                        if text.is_empty() {
                            continue;
                        }

                        let trimmed = text.trim();
                        if trimmed == "/quit" || trimmed == "/exit" {
                            app.should_quit = true;
                            continue;
                        }

                        if trimmed == "/reset" {
                            app.waiting = true;
                            let _ = cmd_tx.send(SessionCmd::Reset);
                            continue;
                        }

                        app.messages.push(("user".to_string(), text.clone()));
                        app.waiting = true;
                        let _ = cmd_tx.send(SessionCmd::Send(text));
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
