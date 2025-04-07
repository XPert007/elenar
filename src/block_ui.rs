use color_eyre::Result;
use figlet_rs::FIGfont;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect, Size},
    style::{Stylize, palette::tailwind},
    text::{Line, Text},
    widgets::*,
};
use std::io::{self, Write};
use tui_scrollview::{ScrollView, ScrollViewState};

pub fn run_ui(
    novel_name: &str,
    chapter_number: &str,
    chapter_name: &str,
    content: Vec<String>,
) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new(novel_name, chapter_number, chapter_name, content);
    let result = app.run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Clone)]
struct App {
    novel_name: String,
    chapter_number: String,
    chapter_name: String,
    text: Vec<String>,
    scroll_view_state: ScrollViewState,
    state: AppState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Running,
    Quit,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Running
    }
}

impl App {
    fn new(novel_name: &str, chapter_number: &str, chapter_name: &str, text: Vec<String>) -> Self {
        Self {
            novel_name: novel_name.to_string(),
            chapter_number: chapter_number.to_string(),
            chapter_name: chapter_name.to_string(),
            text,
            scroll_view_state: ScrollViewState::default(),
            state: AppState::Running,
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.is_running() {
            self.draw(&mut terminal)?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.state == AppState::Running
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        use KeyCode::*;
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                Char('q') | Esc => self.quit(),
                Char('j') | Down => self.scroll_view_state.scroll_down(),
                Char('k') | Up => self.scroll_view_state.scroll_up(),
                Char('f') | PageDown => self.scroll_view_state.scroll_page_down(),
                Char('b') | PageUp => self.scroll_view_state.scroll_page_up(),
                Char('g') | Home => self.scroll_view_state.scroll_to_top(),
                Char('G') | End => self.scroll_view_state.scroll_to_bottom(),
                _ => (),
            },
            _ => {}
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.state = AppState::Quit;
    }
}

const SCROLLVIEW_HEIGHT: u16 = 100;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [title_area, body_area] = layout.areas(area);

        self.title().render(title_area, buf);
        let width = if buf.area.height < SCROLLVIEW_HEIGHT {
            buf.area.width - 1
        } else {
            buf.area.width
        };
        let mut scroll_view = ScrollView::new(Size::new(width, SCROLLVIEW_HEIGHT));
        self.render_widgets_into_scrollview(scroll_view.buf_mut());
        scroll_view.render(body_area, buf, &mut self.scroll_view_state)
    }
}

impl App {
    fn title(&self) -> impl Widget {
        let palette = tailwind::SLATE;
        let fg = palette.c900;
        let bg = palette.c300;
        let keys_fg = palette.c50;
        let keys_bg = palette.c600;
        Line::from(vec![
            self.novel_name.clone().into(),
            "  ↓ | ↑ | PageDown | PageUp | Home | End  "
                .fg(keys_fg)
                .bg(keys_bg),
            "  Quit: ".into(),
            " Esc ".fg(keys_fg).bg(keys_bg),
        ])
        .style((fg, bg))
    }

    fn render_widgets_into_scrollview(&self, buf: &mut Buffer) {
        use Constraint::*;
        let area = buf.area;
        let [numbers_area, widgets_area] = Layout::horizontal([Length(5), Fill(1)]).areas(area);
        let [vertical_area, horizontal_area, content_area] =
            Layout::vertical([Length(7), Length(7), Fill(1)]).areas(widgets_area);

        self.line_numbers(area.height).render(numbers_area, buf);
        self.vertical_bar_chart().render(vertical_area, buf);
        self.horizontal_bar_chart().render(horizontal_area, buf);
        self.texts().render(content_area, buf);
    }

    fn line_numbers(&self, height: u16) -> impl Widget {
        use std::fmt::Write;
        let line_numbers = (1..=height).fold(String::new(), |mut output, n| {
            let _ = writeln!(output, "{n:>4} ");
            output
        });
        Text::from(line_numbers).dim()
    }

    fn vertical_bar_chart(&self) -> impl Widget {
        let block = Block::bordered().title("CHAPTER NUMBER");
        let font = FIGfont::standard().unwrap();
        let content = font
            .convert(&self.chapter_number)
            .expect("FIGlet conversion failed")
            .to_string();
        Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(block)
    }

    fn horizontal_bar_chart(&self) -> impl Widget {
        let block = Block::bordered().title("CHAPTER NAME");
        let font = FIGfont::standard().unwrap();
        let content = font
            .convert(&self.chapter_name)
            .expect("FIGlet conversion failed")
            .to_string();
        Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(block)
    }

    fn texts(&self) -> impl Widget {
        let combined = self.text.join("\n\n");
        let block = Block::bordered().title("CONTENT");
        Paragraph::new(combined)
            .wrap(Wrap { trim: false })
            .block(block)
    }
}
