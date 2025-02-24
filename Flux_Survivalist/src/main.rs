use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};

use std::{error::Error, io, time::{Duration, Instant}};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

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

struct App {
    scroll: u8,
    inventory: Vec<(Item, u8)>,
}

impl App {
    fn new() -> App {
        App {
            scroll: 0,
            inventory: vec![
                (Item::Wood, 10),
                (Item::Fibre, 3),
                (Item::Water, 13)
            ],
        }
    }

    fn on_tick(&mut self) {
        self.scroll += 1;
        self.scroll %= 10;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

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

fn run_app<B: Backend>(
        terminal: &mut Terminal<B>,
        mut app: App,
        tick_rate: Duration
    ) -> io::Result<()> {
    
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            app.on_tick();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    render_inventory(f, &app.inventory, chunks[1]);

}

fn render_inventory<'a, B: Backend>(f: &mut Frame<B>, inv: &Vec<(Item, u8)>, chunk: Rect) {
    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let mut text = vec![];

    for i in 0..inv.len() {
        text.push(
            Spans::from(vec![
                    Span::styled(inv[i].0.as_str(),
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                    ),
                    Span::raw("     "),
                    Span::styled(inv[i].1.to_string() + "/256", Style::default()
                        .add_modifier(Modifier::ITALIC)
                    ),
                    Span::raw("\n"),
                ]
            )
        );
    }

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .block(create_block("Inventory"))
        .alignment(Alignment::Right);

    f.render_widget(paragraph, chunk);
}