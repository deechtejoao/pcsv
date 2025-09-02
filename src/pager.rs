use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{self, Color},
    terminal::{self, ClearType},
};
use std::io::{self, stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::config::PagerConfig;

#[derive(Debug, Clone)]
pub struct PagerState {
    pub current_page: usize,
    pub total_pages: usize,
    pub rows_per_page: usize,
    pub total_rows: usize,
    pub current_row: usize,
    pub terminal_height: u16,
    pub terminal_width: u16,
}

impl PagerState {
    pub fn new(total_rows: usize) -> io::Result<Self> {
        let (terminal_width, terminal_height) = terminal::size()?;
        // Use full terminal height
        let rows_per_page = terminal_height as usize;
        let total_pages = if rows_per_page > 0 {
            (total_rows + rows_per_page - 1) / rows_per_page
        } else {
            1
        };

        Ok(Self {
            current_page: 0,
            total_pages,
            rows_per_page,
            total_rows,
            current_row: 0,
            terminal_height,
            terminal_width,
        })
    }

    pub fn go_to_page(&mut self, page: usize) {
        self.current_page = page.min(self.total_pages.saturating_sub(1));
        self.current_row = self.current_page * self.rows_per_page;
    }

    pub fn next_page(&mut self) {
        if self.current_page < self.total_pages.saturating_sub(1) {
            self.go_to_page(self.current_page + 1);
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.go_to_page(self.current_page.saturating_sub(1));
        }
    }

    pub fn next_row(&mut self) {
        if self.current_row < self.total_rows.saturating_sub(1) {
            self.current_row += 1;
            self.current_page = self.current_row / self.rows_per_page;
        }
    }

    pub fn prev_row(&mut self) {
        if self.current_row > 0 {
            self.current_row = self.current_row.saturating_sub(1);
            self.current_page = self.current_row / self.rows_per_page;
        }
    }

    pub fn go_to_first(&mut self) {
        self.go_to_page(0);
    }

    pub fn go_to_last(&mut self) {
        self.go_to_page(self.total_pages.saturating_sub(1));
    }

    pub fn get_page_start(&self) -> usize {
        self.current_page * self.rows_per_page
    }

    pub fn get_page_end(&self) -> usize {
        (self.get_page_start() + self.rows_per_page).min(self.total_rows)
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.current_row = (self.current_row + lines).min(self.total_rows.saturating_sub(1));
        // Update page to keep current row visible
        self.current_page = self.current_row / self.rows_per_page;
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.current_row = self.current_row.saturating_sub(lines);
        // Update page to keep current row visible
        self.current_page = self.current_row / self.rows_per_page;
    }

    pub fn get_viewport_start(&self) -> usize {
        self.current_row
    }

    pub fn get_viewport_end(&self) -> usize {
        (self.current_row + self.rows_per_page).min(self.total_rows)
    }
}

pub struct Pager {
    state: PagerState,
    content: Vec<String>,
    header: Option<String>,
    config: PagerConfig,
}

impl Pager {
    pub fn new(content: Vec<String>, header: Option<String>, total_rows: usize, config: PagerConfig) -> io::Result<Self> {
        let state = PagerState::new(total_rows)?;
        Ok(Self {
            state,
            content,
            header,
            config,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;

        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();

        // Spawn input handler thread
        thread::spawn(move || {
            loop {
                if let Ok(event) = event::read() {
                    if tx_clone.send(event).is_err() {
                        break;
                    }
                }
            }
        });

        // Initial render
        self.render()?;

        // Main event loop
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Event::Key(key_event)) => {
                    if self.handle_key_event(key_event)? {
                        break;
                    }
                    self.render()?;
                }
                Ok(Event::Resize(width, height)) => {
                    self.state.terminal_width = width;
                    self.state.terminal_height = height;
                    self.state.rows_per_page = height as usize;
                    self.state.total_pages = if self.state.rows_per_page > 0 {
                        (self.state.total_rows + self.state.rows_per_page - 1) / self.state.rows_per_page
                    } else {
                        1
                    };
                    self.state.go_to_page(self.state.current_page.min(self.state.total_pages.saturating_sub(1)));
                    self.render()?;
                }
                Ok(_) => {}
                Err(_) => {
                    // Timeout, continue
                }
            }
        }

        // Cleanup
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            // Page-based scrolling (like less)
            KeyCode::Char(' ') | KeyCode::PageDown => {
                self.state.scroll_down(self.state.rows_per_page);
            }
            KeyCode::Char('b') | KeyCode::PageUp => {
                self.state.scroll_up(self.state.rows_per_page);
            }
            // Configurable line scrolling
            KeyCode::Char('j') | KeyCode::Down => self.state.scroll_down(self.config.scroll_single_line),
            KeyCode::Char('J') => self.state.scroll_down(self.config.scroll_multi_line),
            KeyCode::Char('k') | KeyCode::Up => self.state.scroll_up(self.config.scroll_single_line),
            KeyCode::Char('K') => self.state.scroll_up(self.config.scroll_multi_line),
            // Half page scrolling
            KeyCode::Char('d') => {
                self.state.scroll_down(self.state.rows_per_page / 2);
            }
            KeyCode::Char('u') => {
                self.state.scroll_up(self.state.rows_per_page / 2);
            }
            // Navigation
            KeyCode::Char('g') => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.state.go_to_first();
                } else {
                    self.state.go_to_first();
                }
            }
            KeyCode::Char('G') => self.state.go_to_last(),
            // Home and End keys
            KeyCode::Home => self.state.go_to_first(),
            KeyCode::End => self.state.go_to_last(),
            KeyCode::Char('/') => {
                // TODO: Implement search functionality
            }
            KeyCode::Char('n') => {
                // TODO: Implement next search result
            }
            KeyCode::Char('N') => {
                // TODO: Implement previous search result
            }
            _ => {}
        }
        Ok(false)
    }

    fn render(&mut self) -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        let mut y = 0;

        // Render header if present
        if let Some(header) = &self.header {
            execute!(stdout(), style::SetForegroundColor(Color::Cyan))?;
            println!("{}", header);
            y += 1;
        }

        // Render content for current viewport
        let start = self.state.get_viewport_start();
        let end = self.state.get_viewport_end();

        for (_i, line) in self.content.iter().enumerate().skip(start).take(end - start) {
            if y >= self.state.terminal_height {
                break;
            }
            execute!(stdout(), cursor::MoveTo(0, y))?;
            println!("{}", line);
            y += 1;
        }

        stdout().flush()?;
        Ok(())
    }


}
