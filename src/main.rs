/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages


#[macro_use]
extern crate log;
extern crate log4rs;

extern crate env_logger;

mod util;

use std::io::{self, Write};

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use log::LevelFilter;
use std::sync::{Arc, Mutex};
use std::thread;

use util::event::{Event, Events};

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: Arc<Mutex<String>>,
    /// History of recorded messages
    messages: Arc<Mutex<Vec<String>>>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: Arc::new(Mutex::new(String::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn render(app: App) -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    // Setup event handlers
    let events = Events::new();
    loop {
        // Draw UI
        {
            let app_input_clone = app.input.clone();
            let mut app_input = app_input_clone.lock().unwrap();

            let app_messages_clone = app.messages.clone();
            let mut app_messages = app_messages_clone.lock().unwrap();

            terminal.draw(|mut f| {

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                    .split(f.size());

                Paragraph::new([Text::raw(&*app_input)].iter())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Input"))
                    .render(&mut f, chunks[0]);

                let messages = app_messages
                    .iter()
                    .enumerate()
                    .map(|(i, m)| Text::raw(format!("{}: {}", i, m)));

                List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Messages"))
                    .render(&mut f, chunks[1]);
            })?;
        }

        {
            let app_input_clone = app.input.clone();
            let mut app_input = app_input_clone.lock().unwrap();

            let app_messages_clone = app.messages.clone();
            let mut app_messages = app_messages_clone.lock().unwrap();

            // Put the cursor back inside the input box
            write!(
                terminal.backend_mut(),
                "{}",
                Goto(4 + app_input.width() as u16, 4)
            )?;

            // Handle input
            match events.next()? {
                Event::Input(input) => match input {
                    Key::Char('q') => {
                        break;
                    }
                    Key::Char('\n') => {
                        app_messages.push(app_input.drain(..).collect());
                    }
                    Key::Char(c) => {
                        app_input.push(c);
                    }
                    Key::Backspace => {
                        app_input.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
            info!("{}", app_input);
        }

        
    }
    Ok(())
}


fn main() -> Result<(), failure::Error> {

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/scd.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
                    .appender("logfile")
                    .build(LevelFilter::Info))?;

    log4rs::init_config(config)?;


    // Create default app state
    let mut app = App::default();

    render(app);

    Ok(())
}
