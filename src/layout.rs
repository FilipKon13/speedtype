use std::iter;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::*,
    Frame,
};

struct AppLayout<'a> {
    // main_block, gauge_area, stat_area, text_area
    main_block: Block<'a>,
    gauge_area: Rect,
    stat_area: Rect,
    text_area: Rect,
}

fn get_layout<'a>(frame_size: Rect, text_line_width: u16) -> AppLayout<'a> {
    let main_block = Block::new()
        .borders(Borders::TOP)
        .title(block::Title::from("SpeedType").alignment(Alignment::Center));
    let [gauge_area, _, stat_area, _, text_line, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Percentage(20),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(main_block.inner(frame_size));
    let [_, text_area, _] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(text_line_width),
        Constraint::Min(0),
    ])
    .areas(text_line);
    AppLayout {
        main_block,
        gauge_area,
        stat_area,
        text_area,
    }
}

pub struct TestLine<'a> {
    line: Line<'a>,
    cursor_ind: u16,
}

impl<'a> TestLine<'a> {
    pub fn new(text: &[char], user_text: &[char]) -> Self {
        TestLine {
            line: Line::from_iter(
                text.iter()
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
                    }),
            ),
            cursor_ind: user_text.len() as u16,
        }
    }
    /// use same `area` as in `Frame::render()`
    fn get_cursor(&self, area: Rect) -> (u16, u16) {
        (area.left() + self.cursor_ind, area.top())
    }
}

impl<'a> Widget for TestLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line.render(area, buf)
    }
}

pub fn get_ui_live(
    wpm: u16,
    acc: u16,
    gauge_percent: u16,
    text: TestLine,
) -> impl FnOnce(&mut Frame) + '_ {
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
    move |frame| {
        let AppLayout {
            main_block,
            gauge_area,
            stat_area,
            text_area,
        } = get_layout(frame.size(), text.line.width() as u16);
        frame.render_widget(main_block, frame.size());
        frame.render_widget(gauge, gauge_area);
        frame.render_widget(stat_line, stat_area);
        let (x, y) = text.get_cursor(text_area);
        frame.render_widget(text, text_area);
        frame.set_cursor(x, y);
    }
}

pub fn get_ui_start(text: TestLine) -> impl FnOnce(&mut Frame) + '_ {
    move |frame: &mut Frame| {
        let AppLayout {
            main_block,
            gauge_area: _,
            stat_area: _,
            text_area,
        } = get_layout(frame.size(), text.line.width() as u16);
        frame.render_widget(main_block, frame.size());
        let (x, y) = text.get_cursor(text_area);
        frame.render_widget(text, text_area);
        frame.set_cursor(x, y)
    }
}
