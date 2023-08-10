extern crate serde_json;
extern crate ureq;

use std::{error::Error, io::Stdout, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};

use crate::{
    data::{get_top_cryptos, CryptoPrice},
    utils::format_price,
};

#[derive(Default)]
pub struct AppState {
    pub top_cryptos: Vec<CryptoPrice>,
    pub scroll: u16,
    pub target_scroll: u16,
    pub scroll_area: u16,
    pub scroll_state: ScrollbarState,
}

impl AppState {
    fn set_scroll_area(&mut self, area: u16) {
        self.scroll_area = area;
        self.scroll_state = self
            .scroll_state
            .content_length(self.top_cryptos.len() as u16 - area);
    }

    fn max_position(&self) -> u16 {
        self.top_cryptos.len() as u16 - self.scroll_area
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_position() {
            self.scroll += 1;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }

    pub fn scroll_down_page(&mut self) {
        if self.target_scroll < self.max_position() - self.scroll_area {
            self.target_scroll += self.scroll_area;
        } else {
            self.target_scroll = self.max_position();
        }
    }

    pub fn scroll_up_page(&mut self) {
        if self.target_scroll > self.scroll_area {
            self.target_scroll -= self.scroll_area;
        } else {
            self.target_scroll = 0;
        }
    }

    pub fn animate(&mut self) {
        let diff = self.target_scroll as i32 - self.scroll as i32;

        if diff > 1 {
            self.scroll += (self.target_scroll - self.scroll) / 3;
        }

        if diff.abs() <= 1 {
            self.scroll = self.target_scroll;
        }

        if diff < -1 {
            self.scroll -= (self.scroll - self.target_scroll) / 3;
        }

        self.scroll_state = self.scroll_state.position(self.scroll);
    }
}

pub fn draw_top_cryptos(
    frame: &mut Frame<CrosstermBackend<Stdout>>,
    area: Rect,
    app: &mut AppState,
) {
    let AppState {
        top_cryptos,
        scroll,
        ..
    } = app;
    let area = area.inner(&Margin {
        vertical: 2,
        horizontal: 5,
    });

    let headers = [
        "#",
        "Symbol",
        "Name",
        "Price",
        "Change",
        "Market Cap",
        "Volume (24h)",
    ];

    let remaining_spaces = area.width - 6 - 16;
    let remaining_columns = headers.len() - 2;
    let column_width = remaining_spaces / remaining_columns as u16;
    let widths = [
        Constraint::Max(3),
        Constraint::Max(16),
        Constraint::Max(column_width),
        Constraint::Max(column_width),
        Constraint::Max(column_width),
        Constraint::Max(column_width),
        Constraint::Max(column_width),
    ];

    let header = Row::new(headers)
        .style(Style::default().fg(Color::Yellow))
        .bottom_margin(1);

    let rows: Vec<Row> = top_cryptos
        .iter()
        .enumerate()
        .map(|(index, crypto)| {
            let change_style = Style::default().fg(if crypto.change > 0.0 {
                Color::Green
            } else {
                Color::Red
            });

            let change_formatted = format!("{:.2}%", crypto.change);
            Row::new(vec![
                Line::from((index + 1).to_string()).alignment(Alignment::Right),
                Line::from(crypto.symbol.clone()),
                Line::from(crypto.name.clone()),
                Line::from(format_price(crypto.price)).alignment(Alignment::Right),
                Line::styled(change_formatted, change_style).alignment(Alignment::Right),
                Line::from(format_price(crypto.market_cap)).alignment(Alignment::Right),
                Line::from(format_price(crypto.volume_24h)).alignment(Alignment::Right),
            ])
        })
        .skip(*scroll as usize)
        .collect();

    let table = Table::new(rows)
        .header(header)
        .widths(&widths);

    app.set_scroll_area(area.height - 2u16);

    frame.render_widget(table, area);
    frame.render_stateful_widget(
        Scrollbar::default().style(Style::default().fg(Color::Yellow)),
        area,
        &mut app.scroll_state,
    )
}

pub fn draw_app(frame: &mut Frame<CrosstermBackend<Stdout>>, app: &mut AppState) {
    let size = frame.size();
    let inner = size.inner(&Margin {
        vertical: 3,
        horizontal: 6,
    });

    let block = Block::default()
        .title("CrypTOP")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .border_type(BorderType::Rounded);

    draw_top_cryptos(frame, inner, app);

    frame.render_widget(block, inner);
}

pub fn handle_event(app: &mut AppState) -> Result<bool, Box<dyn Error>> {
    if !event::poll(Duration::from_millis(32))? {
        return Ok(false);
    }

    if let Event::Key(k) = event::read()? {
        match k.code {
            KeyCode::Down => { app.scroll_down(); }
            KeyCode::Up => { app.scroll_up(); }
            KeyCode::Char('r') => { app.top_cryptos = get_top_cryptos(); }
            KeyCode::PageDown => { app.scroll_down_page(); }
            KeyCode::PageUp => { app.scroll_up_page(); }
            KeyCode::Char('q') => { return Ok(true); }
            _ => {}
        }
    };

    Ok(false)
}
