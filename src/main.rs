pub mod app;
pub mod data;
pub mod utils;

use std::{
    error::Error,
    io::{self, Stdout},
};

use anyhow::Result;
use app::{draw_app, handle_event, AppState};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use data::get_top_cryptos;
use ratatui::prelude::*;
extern crate serde_json;
extern crate ureq;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    run(&mut terminal)?;
    restore_terminal(&mut terminal)?;

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;

    Ok(terminal.show_cursor()?)
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let top_cryptos = get_top_cryptos();
    let mut app = AppState {
        top_cryptos,
        ..Default::default()
    };

    loop {
        terminal.draw(|frame| draw_app(frame, &mut app))?;

        if handle_event(&mut app)? {
            break;
        }
    }

    Ok(())
}
