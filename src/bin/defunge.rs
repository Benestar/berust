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

enum InterpreterMessage {
    TogglePause,
    Slower,
    Faster,
    Step,
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

    let int = Arc::new(Mutex::new(interpreter));

    let int2 = Arc::clone(&int);

    let (send, receiver) = mpsc::channel();
    let (int_send, int_receive) = mpsc::channel();

    let send2 = mpsc::Sender::clone(&send);

    thread::spawn(move || {
        interpreter_handler(send2, int_receive, int2);
    });

    thread::spawn(move || {
        input_handler(send);
    });

    // ---

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(AlternateScreen::from(stdout));
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    'outer: loop {
        terminal.draw(|mut f| {
            let mut text = Vec::new();

            let interpreter = int.lock().unwrap();

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

        for ev in receiver.try_iter() {
            if let Event::Input(k) = ev {
                match k {
                    Key::Char('q') => break,
                    Key::Char('p') => int_send.send(InterpreterMessage::TogglePause).unwrap(),
                    Key::Char('n') => int_send.send(InterpreterMessage::Step).unwrap(),
                    Key::Left => int_send.send(InterpreterMessage::Slower).unwrap(),
                    Key::Right => int_send.send(InterpreterMessage::Faster).unwrap(),
                    _ => (),
                }

                continue 'outer
            }
        }

        if let Event::Input(k) = receiver.recv().unwrap() {
            match k {
                Key::Char('q') => break,
                Key::Char('p') => int_send.send(InterpreterMessage::TogglePause).unwrap(),
                Key::Char('n') => int_send.send(InterpreterMessage::Step).unwrap(),
                Key::Left => int_send.send(InterpreterMessage::Slower).unwrap(),
                Key::Right => int_send.send(InterpreterMessage::Faster).unwrap(),
                _ => (),
            }
        }
    }

    Ok(())
}

fn interpreter_handler(
    sender: mpsc::Sender<Event<Key>>,
    receiver: mpsc::Receiver<InterpreterMessage>,
    interpreter: Arc<Mutex<Interpreter<impl Read, impl Write>>>,
) {
    let mut delay = 100;
    let mut running = true;

    loop {
        let start = Instant::now();

        for msg in receiver.try_iter() {
            match msg {
                InterpreterMessage::TogglePause => running = !running,
                InterpreterMessage::Slower => delay = cmp::min(delay + (delay / 5), 1000),
                InterpreterMessage::Faster => delay = cmp::max(delay - (delay / 5), 10),
                InterpreterMessage::Step if !running => {
                    interpreter.lock().unwrap().next();

                    sender.send(Event::Tick).unwrap();
                }
                _ => (),
            }
        }

        if running {
            interpreter.lock().unwrap().next();

            sender.send(Event::Tick).unwrap();
        }

        if let Some(d) = Duration::from_millis(delay).checked_sub(start.elapsed()) {
            thread::sleep(d);
        }
    }
}

fn input_handler(
    sender: mpsc::Sender<Event<Key>>
) {
    let stdin = io::stdin();

    for result in stdin.keys() {
        if let Ok(key) = result {
            sender.send(Event::Input(key)).unwrap();
        }
    }
}
