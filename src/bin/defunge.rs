extern crate berust;
extern crate tui;

use berust::interpreter::{InputOutput, Interpreter};
use berust::playfield::Playfield;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

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
    //let input = Vec::new();
    let output = Vec::new();
    let io = InputOutput::new(std::io::stdin(), output);

    let mut interpreter = Interpreter::new(playfield, io);

    // ---

    let stdout = io::stdout().into_raw_mode()?;
    let stdin = termion::async_stdin();
    let mut keys = stdin.keys();
    let backend = TermionBackend::new(AlternateScreen::from(stdout));
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    loop {
        terminal.draw(|mut f| {
            let mut text = Vec::new();

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

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(interpreter.field().dimensions().0 as u16 + 4),
                        Constraint::Min(0),
                    ].as_ref(),
                )
                .split(f.size());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(interpreter.field().dimensions().1 as u16 + 2),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(cols[0]);

            Paragraph::new(text.iter())
                .block(playfield)
                .alignment(Alignment::Center)
                .render(&mut f, chunks[0]);

            let text = vec![Text::raw(format!("{:?}", interpreter.stack()))];

            Paragraph::new(text.iter())
                .block(stack)
                .alignment(Alignment::Left)
                .render(&mut f, chunks[1]);

            let text = vec![Text::raw(std::str::from_utf8(interpreter.io().writer()).unwrap())];

            Paragraph::new(text.iter())
                .block(output)
                .alignment(Alignment::Left)
                .render(&mut f, cols[1]);
        })?;

        if let Some(Ok(Key::Char('q'))) = keys.next() {
            break;
        }

        interpreter.next();

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}
