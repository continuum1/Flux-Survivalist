use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen}
};

use std::{error::Error, io, rc::Rc, time::{Duration, Instant}};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use tree_ds::prelude::{
    Tree,
    Node,
};

#[derive(Copy, Clone)]
enum Item {
    Wood,
    Fibre,
    Water,
}

impl Item {
    fn as_str(&self) -> &'static str {
        match self {
            Item::Wood => "wood",
            Item::Fibre => "fibre",
            Item::Water => "water",
        }
    }
}

struct AppData {
    inventory: Vec<(Item, u8)>,
}

impl AppData {
    fn new() -> AppData {
        AppData {
            inventory: vec![
                (Item::Wood, 10),
                (Item::Fibre, 3),
                (Item::Water, 13)
            ],
        }
    }
}

struct Frontend<'a> {
    pub titles: Vec<&'a str>,
    cur_tab: usize,
    scroll: u8,
}

impl<'a> Frontend<'a> {
    fn new() -> Frontend<'a> {
        Frontend {
            titles: vec![
                "Dialog page",
                "Inventory",
                "Crafting",
                "Log",
            ],
            cur_tab: 0,
            scroll: 0,
        }
    }

    fn next(&mut self) {
        self.cur_tab = (self.cur_tab + 1) % self.titles.len()
    }

    fn previous(&mut self) {
        if self.cur_tab > 0 {
            self.cur_tab -= 1;
        } else {
            self.cur_tab = self.titles.len() - 1;
        }
    }

    fn on_tick(&mut self) {
        self.scroll += 1;
        self.scroll %= 10;
    }
}

struct Page<'a, B: Backend> {
    f: Frame<'a, B>,
    pub title: &'a str,
    render: dyn Fn(&mut Frame<B>, &Frontend, &AppData, Rect),
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let frontend = Frontend::new();
    let appdata = AppData::new();
    let res = run_app(&mut terminal, frontend, appdata, tick_rate);

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

fn build_inv_page<'a, B: Backend> (f: Frame<B>) -> Tree<i32, Page<'a, B>> {
    let mut tree: Tree<i32, Page<'a, B>> = Tree::new(Some("Inventory"));
}

fn run_app<B: Backend>(
        terminal: &mut Terminal<B>,
        mut frontend: Frontend,
        mut appdata: AppData,
        tick_rate: Duration
    ) -> io::Result<()> {
    
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &frontend, &appdata))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => frontend.next(),
                    KeyCode::Left => frontend.previous(),
                    _ => {},
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            frontend.on_tick();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, frontend: &Frontend, appdata: &AppData) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));
    f.render_widget(block, size);

    let titles = frontend
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(frontend.cur_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    f.render_widget(tabs, chunks[0]);
    
    let inner = match frontend.cur_tab {
        0 => Block::default().title("Inner 0").borders(Borders::ALL),
        1 => Block::default().title("Inner 1").borders(Borders::ALL),
        2 => Block::default().title("Inner 2").borders(Borders::ALL),
        3 => Block::default().title("Inner 3").borders(Borders::ALL),
        _ => unreachable!(),
    };
    f.render_widget(inner, chunks[1]);
}

fn create_block<'b>(title: &'b str) -> Block<'b> {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

fn render_inventory<'a, 'b, B: Backend>(f: &mut Frame<B>, frontend: &Frontend, appdata: &AppData, chunk: Rect) {
    let mut text = vec![];

    for i in 0..appdata.inventory.len() {
        text.push(write_inv_item(appdata.inventory[i]));
    }

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .block(create_block("Inventory"))
        .alignment(Alignment::Right);

    f.render_widget(paragraph, chunk);
}

fn write_inv_item<'a>(item: (Item, u8)) -> Spans<'a> {
    return Spans::from(vec![
            Span::styled(item.0.as_str(),
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
            ),
            Span::raw("     "),
            Span::styled(item.1.to_string() + "/256", Style::default()
                .add_modifier(Modifier::ITALIC)
            ),
            Span::raw("\n"),
        ]
    );
}