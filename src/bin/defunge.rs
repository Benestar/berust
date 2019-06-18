extern crate berust;
extern crate tui;

use berust::interpreter::{InputOutput, Interpreter, Stack};
use berust::playfield::Playfield;
use std::fs::File;
use std::io::{Cursor, Read};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{cmp, env, io, iter, process, str, thread};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

/// An interpreter variant with debug input and output.
pub type DebugInterpreter = Interpreter<Cursor<Vec<u8>>, Vec<u8>>;

/// Either an input event or a simple tick
pub enum Event<I> {
    Input(I),
    Tick,
}

/// A blocking provider of events
///
/// An input event is fired immediately when some user input is registered
/// and a tick event occurs in regular intervals.
pub struct Events {
    receiver: mpsc::Receiver<Event<Key>>,
}

impl Events {
    /// Start separate threads to generate input and tick events.
    ///
    /// A tick event is fired every `1000 / fps` milliseconds.
    pub fn new(fps: u64) -> Self {
        let (sender, receiver) = mpsc::channel();

        {
            // Input thread
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
            // Tick thread
            let sender = sender.clone();

            thread::spawn(move || loop {
                sender.send(Event::Tick).unwrap();
                thread::sleep(Duration::from_millis(1000 / fps));
            });
        }

        Self { receiver }
    }

    /// Block until the next event and return it.
    pub fn next(&self) -> Event<Key> {
        self.receiver.recv().unwrap()
    }
}

/// A message for the runtime environment
///
/// One can tell the runtime to pause or resume, to proceed slower or faster,
/// and, if paused, to execute a single step.
pub enum RuntimeCommand {
    TogglePause,
    Slower,
    Faster,
    Step,
}

/// The runtime environment for an interpreter instance
///
/// It be controlled by sending [`RuntimeCommand`] messages to the runtime.
///
/// [`RuntimeCommand`]: enum.RuntimeCommand.html
pub struct Runtime {
    sender: mpsc::Sender<RuntimeCommand>,
}

impl Runtime {
    /// Start a new thread running the given interpreter.
    pub fn new(interpreter: Arc<Mutex<DebugInterpreter>>) -> Self {
        let (sender, receiver) = mpsc::channel();

        {
            // Runtime thread
            let interpreter = interpreter.clone();

            thread::spawn(move || {
                let mut delay = 100;
                let mut running = false;

                loop {
                    let start = Instant::now();

                    for cmd in receiver.try_iter() {
                        match cmd {
                            RuntimeCommand::TogglePause => running = !running,
                            RuntimeCommand::Slower => delay = cmp::min(delay + (delay / 5), 1000),
                            RuntimeCommand::Faster => delay = cmp::max(delay - (delay / 5), 10),
                            RuntimeCommand::Step if !running => {
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

        Self { sender }
    }

    /// Send a command to the runtime environment.
    pub fn send(&self, cmd: RuntimeCommand) {
        self.sender.send(cmd).unwrap()
    }
}

/// User interface to render the interpreter
pub struct UserInterface {
    terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<io::Stdout>>>>,
    interpreter: Arc<Mutex<DebugInterpreter>>,
}

impl UserInterface {
    /// Create a new UI for the given interpreter.
    pub fn new(interpreter: Arc<Mutex<DebugInterpreter>>) -> io::Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(AlternateScreen::from(stdout));
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        Ok(Self {
            terminal,
            interpreter,
        })
    }

    /// Render the current state of the interpreter.
    pub fn render(&mut self) -> io::Result<()> {
        let interpreter = self.interpreter.lock().unwrap();

        let width = interpreter.field().width();
        let height = interpreter.field().height();

        let playfield = Self::format_playfield(interpreter.field(), interpreter.nav().pos());
        let stack = Self::format_stack(interpreter.stack());
        let output = Self::format_output(interpreter.io().writer());
        let input = Self::format_input(interpreter.io().reader().get_ref());

        self.terminal.draw(|mut f| {
            // -- define layout
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(width as u16 + 4),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(height as u16 + 2), Constraint::Min(0)].as_ref())
                .split(cols[0]);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(cols[1]);

            // -- render blocks and paragraphs
            Paragraph::new(playfield.iter())
                .block(Block::default().title(" Playfield ").borders(Borders::ALL))
                .alignment(Alignment::Center)
                .render(&mut f, left[0]);

            Paragraph::new(stack.iter())
                .block(Block::default().title(" Stack ").borders(Borders::ALL))
                .wrap(true)
                .alignment(Alignment::Left)
                .render(&mut f, left[1]);

            Paragraph::new(output.iter())
                .block(Block::default().title(" Output ").borders(Borders::ALL))
                .alignment(Alignment::Left)
                .render(&mut f, right[0]);

            Paragraph::new(input.iter())
                .block(Block::default().title(" Input ").borders(Borders::ALL))
                .alignment(Alignment::Left)
                .render(&mut f, right[1]);
        })
    }

    fn format_playfield(playfield: &Playfield, pos: (usize, usize)) -> Vec<Text> {
        playfield
            .lines()
            .enumerate()
            .flat_map(move |(y, l)| {
                l.chunks(1)
                    .enumerate()
                    .map(move |(x, c)| {
                        let data = str::from_utf8(c).unwrap();

                        let style = match c[0] {
                            // current position
                            _ if pos == (x, y) => Style::default().bg(Color::Red).fg(Color::White),
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
                            // others
                            _ => Style::default(),
                        };

                        Text::styled(data, style)
                    })
                    .chain(iter::once(Text::raw("\n")))
            })
            .collect()
    }

    fn format_stack(stack: &Stack) -> [Text; 1] {
        [Text::raw(format!("{:?}", stack))]
    }

    fn format_output(output: &[u8]) -> [Text; 1] {
        [Text::raw(str::from_utf8(output).unwrap())]
    }

    fn format_input(input: &[u8]) -> [Text; 1] {
        [Text::raw(str::from_utf8(input).unwrap())]
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./defunge <file>");

        process::exit(1);
    }

    // obtain the interpreter
    let mut file = File::open(&args[1])?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let playfield = Playfield::new(&contents);
    let input = Cursor::new(Vec::new());
    let output = Vec::new();
    let io = InputOutput::new(input, output);

    let interpreter = Interpreter::new(playfield, io);
    let arc = Arc::new(Mutex::new(interpreter));

    // start the event queue and the runtime environment
    let events = Events::new(30);
    let runtime = Runtime::new(Arc::clone(&arc));

    // prepare the terminal
    let mut ui = UserInterface::new(arc)?;

    // start the rendering loop
    loop {
        ui.render()?;

        if let Event::Input(k) = events.next() {
            match k {
                Key::Char('q') => break,
                Key::Char('p') => runtime.send(RuntimeCommand::TogglePause),
                Key::Char('n') => runtime.send(RuntimeCommand::Step),
                Key::Left => runtime.send(RuntimeCommand::Slower),
                Key::Right => runtime.send(RuntimeCommand::Faster),
                _ => (),
            }
        }
    }

    Ok(())
}
