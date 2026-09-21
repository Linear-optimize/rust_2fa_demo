use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout,  Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Gauge, Padding, Paragraph},
};

use crate::totp;

const CARD_WIDTH: u16 = 62;
const CARD_HEIGHT: u16 = 19;

const BACKGROUND: Color = Color::Rgb(8, 12, 22);
const CARD_BACKGROUND: Color = Color::Rgb(14, 21, 35);
const MUTED: Color = Color::Rgb(120, 135, 155);
const ACCENT: Color = Color::Rgb(80, 200, 255);

pub fn run(secret: &[u8]) -> io::Result<()> {
    ratatui::run(|terminal| {
        loop {
            let timestamp = current_timestamp()?;
            let code = totp::generate(secret, timestamp);
            let remaining = totp::remaining_seconds(timestamp);

            terminal.draw(|frame| {
                let area = frame.area();

                // 整个终端的背景
                frame.render_widget(Block::new().style(Style::default().bg(BACKGROUND)), area);

                let card_area = centered_area(area);
                frame.render_widget(Clear, card_area);

                render_card(frame, card_area, code, remaining);
            })?;

            if should_quit()? {
                return Ok(());
            }
        }
    })
}

fn render_card(frame: &mut ratatui::Frame, area: Rect, code: u32, remaining: u64) {
      if area.width < 45 || area.height < CARD_HEIGHT {
        let code_text = if area.width < 10 {
            format!("{code:06}")
        } else {
            format_code(code)
        };

        let mut lines = vec![Line::from(code_text)];
        if area.height >= 2 {
            lines.push(Line::from(format!("剩余 {remaining} 秒")));
        }

        let mut compact = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(status_color(remaining)));

        // 有足够空间才画边框，避免边框挤掉验证码。
        if area.width >= 10 && area.height >= 5 {
            compact = compact.block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" TOTP "),
            );
        }

        frame.render_widget(compact, area);
        return;
    }
    
    let color = status_color(remaining);
    let status = status_text(remaining);

    let card = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(Color::White).bg(CARD_BACKGROUND))
        .padding(Padding::horizontal(2))
        .title_top(
            Line::from(vec![
                Span::styled(" ◆ ", Style::default().fg(ACCENT)),
                Span::styled(
                    "RUST TOTP",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ◆ ", Style::default().fg(ACCENT)),
            ])
            .centered(),
        )
        .title_bottom(
            Line::from(" q / Esc 退出 ")
                .style(Style::default().fg(MUTED))
                .right_aligned(),
        );

    let inner = card.inner(area);
    frame.render_widget(card, area);

    let [
        header_area,
        status_area,
        otp_area,
        separator_area,
        gauge_area,
        footer_area,
    ] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(2),
    ])
    .areas(inner);

    let header = Paragraph::new(
        Line::from("Two-Factor Authentication")
            .style(Style::default().fg(MUTED).add_modifier(Modifier::ITALIC)),
    )
    .alignment(Alignment::Center);

    let status_line = Paragraph::new(
        Line::from(vec![
            Span::styled(
                "● ",
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(status, Style::default().fg(color)),
        ])
        .centered(),
    );

    let otp = Paragraph::new(format_code(code))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(color)
                .bg(Color::Rgb(19, 29, 47))
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(color))
                .padding(Padding::vertical(1))
                .title(
                    Line::from(" CURRENT CODE ")
                        .style(Style::default().fg(MUTED))
                        .centered(),
                ),
        );

    let separator = Paragraph::new(
        Line::from("·  ·  ·  ·  ·  ·  ·  ·  ·  ·").style(Style::default().fg(Color::DarkGray)),
    )
    .alignment(Alignment::Center);

    let progress = Gauge::default()
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(" VALIDITY ").style(Style::default().fg(MUTED))),
        )
        .gauge_style(
            Style::default()
                .fg(color)
                .bg(Color::Rgb(30, 40, 55))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(remaining as f64 / 30.0)
        .label(format!("{remaining:02} seconds"));

    let footer = Paragraph::new(vec![
        Line::from("验证码每 30 秒自动刷新").style(Style::default().fg(MUTED)),
        Line::from("请勿向任何人泄露你的密钥").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(header, header_area);
    frame.render_widget(status_line, status_area);
    frame.render_widget(otp, otp_area);
    frame.render_widget(separator, separator_area);
    frame.render_widget(progress, gauge_area);
    frame.render_widget(footer, footer_area);
}

fn centered_area(area: Rect) -> Rect {
      let width = CARD_WIDTH.min(area.width);
    let height = CARD_HEIGHT.min(area.height);

    let [vertical] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);

    let [centered] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(vertical);

    centered
}


fn format_code(code: u32) -> String {
    let code = format!("{code:06}");

    format!("{}  {}", &code[..3], &code[3..])
}

fn status_color(remaining: u64) -> Color {
    match remaining {
        0..=5 => Color::Red,
        6..=10 => Color::Yellow,
        _ => Color::Green,
    }
}

fn status_text(remaining: u64) -> &'static str {
    match remaining {
        0..=5 => "即将刷新",
        6..=10 => "请尽快使用",
        _ => "验证码有效",
    }
}

fn current_timestamp() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(io::Error::other)
}

fn should_quit() -> io::Result<bool> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(false);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };

    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    Ok(matches!(
        key.code,
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
    ))
}
