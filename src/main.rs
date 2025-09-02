use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Serialize, Deserialize, Debug)]
struct TuiConfig {
    title: String,
    border: bool,
    margin: u16,
    content: Vec<ContentItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContentItem {
    #[serde(rename = "type")]
    item_type: String,
    text: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    style: Option<Vec<String>>,
}

fn load_config() -> Result<TuiConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("tui_config.json")?;
    let config: TuiConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}

fn parse_color(color_str: &Option<String>) -> Color {
    match color_str.as_deref() {
        Some("red") => Color::Red,
        Some("green") => Color::Green,
        Some("blue") => Color::Blue,
        Some("yellow") => Color::Yellow,
        Some("cyan") => Color::Cyan,
        Some("magenta") => Color::Magenta,
        Some("gray") | Some("grey") => Color::Gray,
        Some("white") => Color::White,
        _ => Color::White,
    }
}

fn parse_style_modifiers(styles: &Option<Vec<String>>) -> Modifier {
    let mut modifier = Modifier::empty();
    if let Some(style_vec) = styles {
        for style in style_vec {
            match style.as_str() {
                "bold" => modifier |= Modifier::BOLD,
                "italic" => modifier |= Modifier::ITALIC,
                "underlined" => modifier |= Modifier::UNDERLINED,
                _ => {}
            }
        }
    }
    modifier
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, config: TuiConfig) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, &config))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn ui(frame: &mut Frame, config: &TuiConfig) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(config.margin)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(frame.area());

    let mut lines = Vec::new();
    
    for item in &config.content {
        if item.text.is_empty() {
            lines.push(Line::from(Span::raw("")));
        } else {
            let color = parse_color(&item.color);
            let modifier = parse_style_modifiers(&item.style);
            let style = Style::default().fg(color).add_modifier(modifier);
            lines.push(Line::from(Span::styled(&item.text, style)));
        }
    }

    let mut block = Block::default().title(config.title.as_str());
    if config.border {
        block = block.borders(Borders::ALL);
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, chunks[0]);
}
