extern crate berust;
extern crate tui;

use berust::interpreter::{InputOutput, Interpreter};
use berust::playfield::Playfield;
use std::cmp;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Write};
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

enum Event<I> {
    Input(I),
    Tick,
}

struct Events {
    receiver: mpsc::Receiver<Event<Key>>,
}

impl Events {
    fn new() -> Events {
        let (sender, receiver) = mpsc::channel();

        {
            let sender = sender.clone();

            thread::spawn(move || {
                let stdin = io::stdin();

                for result in stdin.keys() {
                    if let Ok(key) = result {
                        sender.send(Event::Input(key)).unwrap();
                    }
                }
            });
        }

        {
            let sender = sender.clone();

            thread::spawn(move || loop {
                sender.send(Event::Tick).unwrap();
                thread::sleep(Duration::from_millis(33));
            });
        }

        Self { receiver }
    }

    fn next(&self) -> Event<Key> {
        self.receiver.recv().unwrap()
    }
}

enum RuntimeMessage {
    TogglePause,
    Slower,
    Faster,
    Step,
}

struct Runtime<R, W> {
    interpreter: Arc<Mutex<Interpreter<R, W>>>,
    sender: mpsc::Sender<RuntimeMessage>,
}

impl<R: Read + Send + 'static, W: Write + Send + 'static> Runtime<R, W> {
    fn new(interpreter: Interpreter<R, W>) -> Self {
        let interpreter = Arc::new(Mutex::new(interpreter));
        let (sender, receiver) = mpsc::channel();

        {
            let interpreter = interpreter.clone();

            thread::spawn(move || {
                let mut delay = 100;
                let mut running = true;

                loop {
                    let start = Instant::now();

                    for msg in receiver.try_iter() {
                        match msg {
                            RuntimeMessage::TogglePause => running = !running,
                            RuntimeMessage::Slower => delay = cmp::min(delay + (delay / 5), 1000),
                            RuntimeMessage::Faster => delay = cmp::max(delay - (delay / 5), 10),
                            RuntimeMessage::Step if !running => {
                                interpreter.lock().unwrap().next().unwrap_or(())
                            }
                            _ => (),
                        }
                    }

                    if running {
                        interpreter.lock().unwrap().next();
                    }

                    if let Some(d) = Duration::from_millis(delay).checked_sub(start.elapsed()) {
                        thread::sleep(d);
                    }
                }
            });
        }

        Self {
            interpreter,
            sender,
        }
    }

    fn interpreter(&self) -> std::sync::MutexGuard<Interpreter<R, W>> {
        self.interpreter.lock().unwrap()
    }

    fn send(&self, message: RuntimeMessage) {
        self.sender.send(message).unwrap()
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./befunge <file>");

        process::exit(1);
    }

    let mut file = File::open(&args[1]).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let playfield = Playfield::new(&contents);
    let input = Cursor::new(Vec::new());
    let output = Vec::new();
    let io = InputOutput::new(input, output);

    let interpreter = Interpreter::new(playfield, io);

    // ---

    let events = Events::new();
    let runtime = Runtime::new(interpreter);

    // ---

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(AlternateScreen::from(stdout));
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    loop {
        terminal.draw(|mut f| {
            let mut text = Vec::new();

            let interpreter = runtime.interpreter();

            for (y, l) in interpreter.field().lines().enumerate() {
                for (x, c) in l.chunks(1).enumerate() {
                    let data = std::str::from_utf8(c).unwrap();

                    let mut style = match c[0] {
                        // numbers
                        b'0'...b'9' => Style::default().fg(Color::Blue),
                        // operators
                        b'+' | b'-' | b'*' | b'/' | b'%' | b'!' | b'`' => {
                            Style::default().fg(Color::Red)
                        }
                        // movement
                        b'>' | b'<' | b'^' | b'v' | b'?' => Style::default().fg(Color::Red),
                        // branching
                        b'_' | b'|' | b'#' | b'@' => Style::default().fg(Color::Red),
                        // stack
                        b':' | b'\\' | b'$' | b'"' => Style::default(),
                        // io
                        b'.' | b',' | b'&' | b'~' => Style::default(),
                        // storage
                        b'p' | b'g' => Style::default().fg(Color::Red),
                        _ => Style::default(),
                    };

                    if interpreter.nav().pos() == (x, y) {
                        style = style.bg(Color::Red).fg(Color::White);
                    }

                    text.push(Text::styled(data, style));
                }

                text.push(Text::raw("\n"));
            }

            let playfield = Block::default().title(" Playfield ").borders(Borders::ALL);

            let stack = Block::default().title(" Stack ").borders(Borders::ALL);

            let output = Block::default().title(" Output ").borders(Borders::ALL);

            let input = Block::default().title(" Input ").borders(Borders::ALL);

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(interpreter.field().dimensions().0 as u16 + 4),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(interpreter.field().dimensions().1 as u16 + 2),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(cols[0]);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(cols[1]);

            Paragraph::new(text.iter())
                .block(playfield)
                .alignment(Alignment::Center)
                .render(&mut f, left[0]);

            let text = vec![Text::raw(format!("{:?}", interpreter.stack()))];

            Paragraph::new(text.iter())
                .block(stack)
                .wrap(true)
                .alignment(Alignment::Left)
                .render(&mut f, left[1]);

            let text = vec![Text::raw(
                std::str::from_utf8(interpreter.io().writer()).unwrap(),
            )];

            Paragraph::new(text.iter())
                .block(output)
                .alignment(Alignment::Left)
                .render(&mut f, right[0]);

            let text = vec![Text::raw(
                std::str::from_utf8(interpreter.io().reader().get_ref()).unwrap(),
            )];

            Paragraph::new(text.iter())
                .block(input)
                .alignment(Alignment::Left)
                .render(&mut f, right[1]);
        })?;

        if let Event::Input(k) = events.next() {
            match k {
                Key::Char('q') => break,
                Key::Char('p') => runtime.send(RuntimeMessage::TogglePause),
                Key::Char('n') => runtime.send(RuntimeMessage::Step),
                Key::Left => runtime.send(RuntimeMessage::Slower),
                Key::Right => runtime.send(RuntimeMessage::Faster),
                _ => (),
            }
        }
    }

    Ok(())
}
