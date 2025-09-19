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
use std::{fs, io};

mod lua_dom;
mod lua_engine;
mod dom_api;

use lua_dom::LuaDOM;
use lua_engine::LuaEngine;


fn load_lua_dom() -> Result<LuaDOM, Box<dyn std::error::Error>> {
    let lua = mlua::Lua::new();

    // Try to load from .luaT file first, fallback to default
    if fs::metadata("dom.luaT").is_ok() {
        LuaDOM::from_lua_file(&lua, "dom.luaT").map_err(|e| e.into())
    } else {
        // Create a default DOM if no .luaT file exists
        Ok(LuaDOM {
            title: "Nebular - Lua Browser".to_string(),
            border: true,
            margin: 1,
            elements: vec![
                lua_dom::DOMElement {
                    element_type: "text".to_string(),
                    text: "Welcome to Nebular!".to_string(),
                    color: Some("blue".to_string()),
                    style: Some(vec!["bold".to_string()]),
                    id: Some("welcome".to_string()),
                },
                lua_dom::DOMElement {
                    element_type: "text".to_string(),
                    text: "This is a Lua-powered TUI browser".to_string(),
                    color: Some("green".to_string()),
                    style: None,
                    id: Some("description".to_string()),
                },
                lua_dom::DOMElement {
                    element_type: "text".to_string(),
                    text: "".to_string(),
                    color: None,
                    style: None,
                    id: None,
                },
                lua_dom::DOMElement {
                    element_type: "text".to_string(),
                    text: "Press 'q' to quit, 'r' to reload scripts".to_string(),
                    color: Some("gray".to_string()),
                    style: None,
                    id: Some("instructions".to_string()),
                },
            ],
        })
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dom = load_lua_dom()?;
    let engine = LuaEngine::new(dom)?;

    // Execute the script.lua file to test DOM manipulation
    if fs::metadata("script.lua").is_ok() {
        if let Err(e) = engine.execute_script_file("script.lua") {
            eprintln!("Error executing script.lua: {}", e);
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, engine);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, engine: LuaEngine) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, &engine))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn ui(frame: &mut Frame, engine: &LuaEngine) {
    let dom_arc = engine.get_dom();
    let dom_guard = match dom_arc.lock() {
        Ok(guard) => guard,
        Err(_) => {
            let error_paragraph = Paragraph::new("Error: Unable to access DOM")
                .block(Block::default().title("Error").borders(Borders::ALL));
            frame.render_widget(error_paragraph, frame.area());
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(dom_guard.margin)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(frame.area());

    let mut lines = Vec::new();

    for element in &dom_guard.elements {
        if element.text.is_empty() {
            lines.push(Line::from(Span::raw("")));
        } else {
            let style = element.get_style();
            lines.push(Line::from(Span::styled(&element.text, style)));
        }
    }

    let mut block = Block::default().title(dom_guard.title.as_str());
    if dom_guard.border {
        block = block.borders(Borders::ALL);
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, chunks[0]);
}
