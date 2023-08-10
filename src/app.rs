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

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll < self.top_cryptos.len() as u16 - self.scroll_area {
            self.scroll += 1;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
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
    let widths = [
        Constraint::Min(3),
        Constraint::Min(6),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
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
            Row::new(vec![
                Cell::from((index + 1).to_string()),
                Cell::from(crypto.symbol.clone()),
                Cell::from(crypto.name.clone()),
                Cell::from(format_price(crypto.price)),
                Cell::from(format!("{:.2}%", crypto.change)).style(change_style),
                Cell::from(format_price(crypto.market_cap)),
                Cell::from(format_price(crypto.volume_24h)),
            ])
            // .bottom_margin(1)
        })
        .skip(*scroll as usize)
        .collect();

    let table = Table::new(rows)
        .header(header)
        .widths(&widths)
        .column_spacing(10);

    app.set_scroll_area(area.height as u16 - 2u16);

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
        .title("Cryptop")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .border_type(BorderType::Rounded);

    draw_top_cryptos(frame, inner, app);

    frame.render_widget(block, inner);
}

pub fn handle_event(app: &mut AppState) -> Result<bool, Box<dyn Error>> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(k) => match k.code {
            KeyCode::Down => {
                app.scroll_down();
            }
            KeyCode::Up => {
                app.scroll_up();
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            KeyCode::Char('r') => {
                app.top_cryptos = get_top_cryptos();
            }
            _ => {}
        },
        _ => {}
    };

    Ok(false)
}
