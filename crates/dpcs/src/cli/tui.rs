//! Interactive TUI inspector for a single Pipeline Contract.

use std::io::{self, IsTerminal, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::Terminal;

use crate::diagnostics::ValidationReport;
use crate::error::{Error, Result};
use crate::parser;
use crate::report::{
    graph_view_from_contract, inspect_view_from_contract, GraphView, InspectView,
};
use crate::validate;

use super::{EXIT_FAILURE, EXIT_OK};

const PANE_TITLES: [&str; 5] = ["Overview", "Steps", "Edges", "Diagnostics", "Plan"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Overview = 0,
    Steps = 1,
    Edges = 2,
    Diagnostics = 3,
    Plan = 4,
}

impl Pane {
    fn from_index(index: usize) -> Self {
        match index % PANE_TITLES.len() {
            0 => Self::Overview,
            1 => Self::Steps,
            2 => Self::Edges,
            3 => Self::Diagnostics,
            _ => Self::Plan,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

struct InspectorState {
    path: PathBuf,
    inspect: InspectView,
    graph: GraphView,
    diagnostics: ValidationReport,
    pane: Pane,
    list_state: ListState,
    status: String,
}

impl InspectorState {
    fn load(path: &Path) -> Result<Self> {
        let contract = parser::parse_file(path)?;
        let inspect = inspect_view_from_contract(&contract);
        let graph = graph_view_from_contract(&contract);
        let diagnostics = validate(&contract);
        Ok(Self {
            path: path.to_path_buf(),
            inspect,
            graph,
            diagnostics,
            pane: Pane::Overview,
            list_state: ListState::default().with_selected(Some(0)),
            status: format!("loaded {}", path.display()),
        })
    }

    fn reload(&mut self) -> Result<()> {
        let next = Self::load(&self.path)?;
        self.inspect = next.inspect;
        self.graph = next.graph;
        self.diagnostics = next.diagnostics;
        self.status = format!("reloaded {}", self.path.display());
        self.reset_selection();
        Ok(())
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn lines_for_pane(&self) -> Vec<String> {
        match self.pane {
            Pane::Overview => {
                let mut lines = vec![
                    format!("id: {}", self.inspect.id),
                    format!("version: {}", self.inspect.version),
                    format!("dpcsVersion: {}", self.inspect.dpcs_version),
                    format!("steps: {}", self.inspect.step_count),
                    format!("edges: {}", self.inspect.edge_count),
                    format!("valid: {}", self.inspect.valid),
                    format!("planningRefused: {}", self.inspect.planning_refused),
                    format!("path: {}", self.path.display()),
                ];
                if let Some(name) = &self.inspect.name {
                    lines.insert(1, format!("name: {name}"));
                }
                lines
            }
            Pane::Steps => {
                if self.inspect.step_ids.is_empty() {
                    vec!["(no steps)".to_owned()]
                } else {
                    self.inspect.step_ids.clone()
                }
            }
            Pane::Edges => {
                if self.graph.edges.is_empty() {
                    vec!["(no edges)".to_owned()]
                } else {
                    self.graph
                        .edges
                        .iter()
                        .map(|e| match &e.kind {
                            Some(kind) => format!("{} -> {} ({kind})", e.from, e.to),
                            None => format!("{} -> {}", e.from, e.to),
                        })
                        .collect()
                }
            }
            Pane::Diagnostics => {
                if self.diagnostics.diagnostics.is_empty() {
                    vec!["valid: no diagnostics".to_owned()]
                } else {
                    self.diagnostics
                        .diagnostics
                        .iter()
                        .map(|d| {
                            format!("{} {}: {} — {}", d.severity, d.id, d.stage, d.message)
                        })
                        .collect()
                }
            }
            Pane::Plan => match &self.inspect.step_order {
                Some(order) if !order.is_empty() => order.clone(),
                Some(_) => vec!["(empty plan)".to_owned()],
                None => vec!["planning: refused".to_owned()],
            },
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.lines_for_pane().len().max(1);
        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        self.list_state.select(Some(next));
    }

    fn set_pane(&mut self, pane: Pane) {
        self.pane = pane;
        self.reset_selection();
    }

    fn next_pane(&mut self) {
        self.set_pane(Pane::from_index(self.pane.index() + 1));
    }
}

/// Run the interactive inspector. Requires a TTY.
pub fn run(path: &Path) -> Result<u8> {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        return Err(Error::Serialization(
            "TUI inspector requires an interactive TTY (stdout and stdin)".to_owned(),
        ));
    }

    let mut state = match InspectorState::load(path) {
        Ok(state) => state,
        Err(Error::InvalidDocument { report }) => {
            eprintln!("error: failed to parse {}; see diagnostics", path.display());
            for diagnostic in &report.diagnostics {
                eprintln!(
                    "{} {}: {} — {}",
                    diagnostic.severity, diagnostic.id, diagnostic.stage, diagnostic.message
                );
            }
            return Ok(EXIT_FAILURE);
        }
        Err(err) => return Err(err),
    };

    let mut terminal = setup_terminal()?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_loop(&mut terminal, &mut state)
    }));
    let restore = restore_terminal(&mut terminal);
    match result {
        Ok(Ok(())) => {
            restore?;
            Ok(EXIT_OK)
        }
        Ok(Err(err)) => {
            let _ = restore;
            Err(err)
        }
        Err(_) => {
            let _ = restore;
            Err(Error::Serialization("TUI inspector panicked".to_owned()))
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().map_err(|err| Error::Serialization(format!("enable raw mode: {err}")))?;
    let mut stdout = io::stdout();
    if let Err(err) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(Error::Serialization(format!(
            "enter alternate screen: {err}"
        )));
    }
    let backend = CrosstermBackend::new(stdout);
    match Terminal::new(backend) {
        Ok(terminal) => Ok(terminal),
        Err(err) => {
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            Err(Error::Serialization(format!("create terminal: {err}")))
        }
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let leave = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let raw = disable_raw_mode();
    let cursor = terminal.show_cursor();
    leave.map_err(|err| Error::Serialization(format!("leave alternate screen: {err}")))?;
    raw.map_err(|err| Error::Serialization(format!("disable raw mode: {err}")))?;
    cursor.map_err(|err| Error::Serialization(format!("show cursor: {err}")))?;
    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut InspectorState,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| draw(frame, state))
            .map_err(|err| Error::Serialization(format!("draw TUI: {err}")))?;

        if !event::poll(Duration::from_millis(250))
            .map_err(|err| Error::Serialization(format!("poll events: {err}")))?
        {
            continue;
        }
        let Event::Key(key) = event::read()
            .map_err(|err| Error::Serialization(format!("read event: {err}")))?
        else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Char('r') => {
                if let Err(err) = state.reload() {
                    state.status = format!("reload failed: {err}");
                }
            }
            KeyCode::Tab | KeyCode::Right => state.next_pane(),
            KeyCode::Left => {
                let idx = state.pane.index() + PANE_TITLES.len() - 1;
                state.set_pane(Pane::from_index(idx));
            }
            KeyCode::Char('1') => state.set_pane(Pane::Overview),
            KeyCode::Char('2') => state.set_pane(Pane::Steps),
            KeyCode::Char('3') => state.set_pane(Pane::Edges),
            KeyCode::Char('4') => state.set_pane(Pane::Diagnostics),
            KeyCode::Char('5') => state.set_pane(Pane::Plan),
            KeyCode::Down | KeyCode::Char('j') => state.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => state.move_selection(-1),
            _ => {}
        }
    }
}

fn draw(frame: &mut ratatui::Frame<'_>, state: &mut InspectorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let titles: Vec<Line<'_>> = PANE_TITLES.iter().map(|t| Line::from(*t)).collect();
    let tabs = Tabs::new(titles)
        .select(state.pane.index())
        .block(Block::default().borders(Borders::ALL).title("dpcs inspect"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    frame.render_widget(tabs, chunks[0]);

    let lines = state.lines_for_pane();
    let items: Vec<ListItem<'_>> = lines.iter().map(|line| ListItem::new(line.clone())).collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(PANE_TITLES[state.pane.index()]),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, chunks[1], &mut state.list_state);

    let help = Paragraph::new(Line::from(vec![
        Span::raw("Tab/1-5 panes  j/k navigate  r reload  q quit  |  "),
        Span::raw(state.status.as_str()),
    ]))
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}
