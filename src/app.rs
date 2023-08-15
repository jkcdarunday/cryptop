extern crate serde_json;
extern crate ureq;

use std::{error::Error, io::Stdout, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
    pub selected: u16,
}

impl AppState {
    fn set_scroll_area(&mut self, area: u16) {
        self.scroll_area = area;
        self.scroll_state = self
            .scroll_state
            .content_length((self.top_cryptos.len() as u16).saturating_sub(area));
    }

    fn max_position(&self) -> u16 {
        (self.top_cryptos.len() as u16).saturating_sub(self.scroll_area)
    }

    fn scroll_to_selected(&mut self) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }

        if self.selected >= self.scroll + self.scroll_area {
            self.scroll = self.selected - self.scroll_area + 1;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }

    fn select_within_view(&mut self) {
        if self.selected < self.scroll {
            self.selected = self.scroll;
        }

        if self.selected >= self.scroll + self.scroll_area {
            self.selected = self.scroll + self.scroll_area - 1;
        }
    }

    pub fn select_up(&mut self) {
        if self.selected == 0 {
            return;
        }

        self.selected -= 1;

        self.scroll_to_selected();
    }

    pub fn select_down(&mut self) {
        if self.selected >= self.top_cryptos.len() as u16 - 1 {
            return;
        }

        self.selected += 1;

        self.scroll_to_selected();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }

        self.select_within_view();
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_position() {
            self.scroll += 1;
            self.target_scroll = self.scroll;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }

        self.select_within_view()
    }

    pub fn scroll_down_page(&mut self) {
        if self.scroll == self.max_position() {
            self.selected = self.top_cryptos.len() as u16 - 1;
        }

        if self.target_scroll < self.max_position().saturating_sub(self.scroll_area) {
            self.target_scroll += self.scroll_area;
        } else {
            self.target_scroll = self.max_position();
        }
    }

    pub fn scroll_up_page(&mut self) {
        if self.scroll == 0 {
            self.selected = 0;
        }

        if self.target_scroll > self.scroll_area {
            self.target_scroll -= self.scroll_area;
        } else {
            self.target_scroll = 0;
        }
    }

    pub fn animate(&mut self) {
        let diff = self.target_scroll as i32 - self.scroll as i32;

        if diff > 2 {
            self.scroll += (self.target_scroll - self.scroll) / 3;
        }

        if diff.abs() <= 2 {
            self.scroll = self.target_scroll;
        }

        if diff < -2 {
            self.scroll -= (self.scroll - self.target_scroll) / 3;
        }

        self.scroll_state = self.scroll_state.position(self.scroll);

        self.select_within_view();
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

    let table_area = area.inner(&Margin {
        vertical: 2,
        horizontal: 5,
    });

    let scrollbar_area = area.inner(&Margin {
        vertical: 2,
        horizontal: 3,
    });

    let headers = vec![
        Line::from("#").alignment(Alignment::Right),
        Line::from("Symbol"),
        Line::from("Name"),
        Line::from("Price").alignment(Alignment::Right),
        Line::from("Change").alignment(Alignment::Right),
        Line::from("Market Cap").alignment(Alignment::Right),
        Line::from("Volume (24h)").alignment(Alignment::Right),
    ];

    let remaining_spaces = table_area.width.saturating_sub(3 + 8);
    let remaining_columns = headers.len().saturating_sub(2);
    let column_spacing = 3;
    let column_spacing_offset = (headers.len() * column_spacing) / remaining_columns;
    let column_width =
        (remaining_spaces / remaining_columns as u16).saturating_sub(column_spacing_offset as u16);
    let widths = [
        Constraint::Max(3),
        Constraint::Max(8),
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
            .style(if app.selected == index as u16 {
                Style::default().fg(Color::Black).bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .skip(*scroll as usize)
        .collect();

    let table = Table::new(rows)
        .header(header)
        .widths(&widths)
        .column_spacing(column_spacing as u16);

    app.set_scroll_area(table_area.height.saturating_sub(2));

    frame.render_widget(table, table_area);
    frame.render_stateful_widget(
        Scrollbar::default().style(Style::default().fg(Color::Yellow)),
        scrollbar_area,
        &mut app.scroll_state,
    )
}

pub fn draw_app(frame: &mut Frame<CrosstermBackend<Stdout>>, app: &mut AppState) {
    let size = frame.size();
    let is_small = size.width <= 160;
    let margin = if is_small {
        Margin {
            vertical: 0,
            horizontal: 1,
        }
    } else {
        Margin {
            vertical: 3,
            horizontal: 9,
        }
    };

    let inner = size.inner(&margin);

    let block = Block::default()
        .title(" CrypTOP ")
        .title_alignment(Alignment::Center)
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
            KeyCode::Down => match k.modifiers {
                KeyModifiers::SHIFT => {
                    app.scroll_down();
                }
                _ => {
                    app.select_down();
                }
            },
            KeyCode::Up => match k.modifiers {
                KeyModifiers::SHIFT => {
                    app.scroll_up();
                }
                _ => {
                    app.select_up();
                }
            },
            KeyCode::Char('r') => {
                app.top_cryptos = get_top_cryptos()?;
            }
            KeyCode::PageDown => {
                app.scroll_down_page();
            }
            KeyCode::PageUp => {
                app.scroll_up_page();
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            _ => {}
        }
    };

    Ok(false)
}
