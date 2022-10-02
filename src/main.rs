/*
Features:
0) Implement program of my variant
1) Create process using winapi
2) Create process using fork()
3) Process afinity selection
4) Display CPU time
5) Implement start, stop and kill for every process started
6) Change priority functionality
*/

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

// let launched_processes: Vec<&Process>;

// if let Ok(exec_path) = std::env::current_exe() {
//     all_processes.values().for_each(|process| {
//         if let Ok(parent_pid) = process.parent() || parent_pid == std::pr {
//             launched_processes.append(process);
//         }
//     });
// }

struct Process<'a> {
    pid: u32,
    name: &'a str,
    command: Option<&'a str>,
    runtime: u64,
}

impl<'a> From<&'a sysinfo::Process> for Process<'a> {
    fn from(sproc: &'a sysinfo::Process) -> Process<'a> {
        Process {
            pid: sproc.pid().as_u32(),
            name: sproc.name(),
            command: sproc.cmd().last().map(|s| s.as_str()),
            runtime: sproc.run_time(),
        }
    }
}

struct App<'a> {
    table: TableState,
    processes: Vec<Process<'a>>,
}

impl<'a> App<'a> {
    fn new(system: &'a System) -> App<'a> {
        App {
            // TODO: fetch only processes data
            processes: system.processes().values().map(Process::from).collect(),
            table: TableState::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.table.selected() {
            Some(i) => {
                if i >= self.processes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table.selected() {
            Some(i) => {
                if i == 0 {
                    self.processes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table.select(Some(i));
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
    let system = System::new_all();
    let app = App::new(&system);
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["PID", "Name", "Command", "Run Time"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.processes.iter().map(|process| {
        let cells = [
            Cell::from(process.pid.to_string()),
            Cell::from(process.name),
            Cell::from(process.command.unwrap_or("N/A")),
            Cell::from(process.runtime.to_string()),
        ];

        Row::new(cells).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("1000 - 7"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.table);
}
