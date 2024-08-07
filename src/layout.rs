use std::iter;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::*,
};

pub struct AppLayout {
    pub gauge_area: Rect,
    pub stat_area: Rect,
    pub text_area: Rect,
}

impl AppLayout {
    pub fn new(frame_size: Rect) -> Self {
        use Constraint::*;
        let [gauge_area, _, stat_area, _, text_lines, _] = Layout::vertical([
            Length(1),
            Length(1),
            Length(1),
            Percentage(20),
            Length(2),
            Fill(1),
        ])
        .areas(frame_size);
        let text_area =
            Layout::horizontal([Fill(1), Percentage(50), Fill(1)]).areas::<3>(text_lines)[1];
        AppLayout {
            gauge_area,
            stat_area,
            text_area,
        }
    }
}

pub struct TestLines<'a> {
    line: Line<'a>,
    next_line: Line<'a>,
}

impl<'a> TestLines<'a> {
    pub fn new(line: &[char], next_line: &[char], user_text: &[char]) -> Self {
        TestLines {
            line: line
                .iter()
                .zip(user_text.iter().map(Some).chain(iter::repeat(None)))
                .map(|(c, u)| {
                    let span = Span::raw(c.to_string());
                    match u {
                        Some(u) => {
                            if c == u {
                                span.green()
                            } else {
                                span.blue().on_red()
                            }
                        }
                        None => span.blue(),
                    }
                })
                .collect(),
            next_line: next_line
                .iter()
                .map(|c| Span::raw(c.to_string()).blue())
                .collect(),
        }
    }
}

impl<'a> Widget for TestLines<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let top = Rect { height: 1, ..area };
        let bot = Rect {
            height: 1,
            y: area.y + 1,
            ..area
        };
        self.line.render(top, buf);
        self.next_line.render(bot, buf);
    }
}

pub struct StartScreen;

impl Widget for &StartScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Line::raw("Press Tab to start")
            .bold()
            .centered()
            .render(area, buf)
    }
}

pub struct GameStatsScreen {
    wpm: f64,
    acc: f64,
}

impl GameStatsScreen {
    pub fn new(wpm: f64, acc: f64) -> Self {
        GameStatsScreen { wpm, acc }
    }
}

impl Widget for &GameStatsScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let AppLayout {
            gauge_area,
            stat_area,
            text_area,
        } = AppLayout::new(area);
        Line::raw("Time ended!")
            .bold()
            .centered()
            .render(gauge_area, buf);
        Line::raw("Press Tab to restart or Esc to quit")
            .bold()
            .centered()
            .render(stat_area, buf);
        let [top_line, bot_line] = Layout::vertical([Length(1), Length(1)]).areas(text_area);
        Line::raw(format!("WPM: {}", self.wpm))
            .bold()
            .centered()
            .render(top_line, buf);
        Line::raw(format!("Accuracy: {}", self.acc))
            .bold()
            .centered()
            .render(bot_line, buf);
    }
}

pub fn get_ui_live_widgets<'a>(
    wpm: usize,
    acc: usize,
    gauge_percent: u16,
) -> (Gauge<'a>, Line<'a>) {
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Blue).bg(Color::Red))
        .percent(gauge_percent)
        .label(Span::default())
        .use_unicode(true);
    let stat_line = Line::from(vec![
        "WPM: ".bold(),
        wpm.to_string().into(),
        " Acc: ".bold(),
        acc.to_string().into(),
    ])
    .left_aligned();
    (gauge, stat_line)
}
