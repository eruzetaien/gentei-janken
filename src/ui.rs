use std::collections::VecDeque;
use std::time::Duration;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use std::io;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style,
    terminal::{self, Clear, ClearType},
};


pub struct PlayerDisplay {
    pub name: String,
    pub status: String,
}

pub enum KeyResult {
    Submitted(String),
    Exit,
}

#[derive(Default)]
struct TermRegion {
    row: u16,
    col: u16,
    width: u16,
    height: u16,
}

pub struct TermUi {
    width: u16,
    height: u16,

    // Computed regions
    title:       TermRegion,
    subtitle:    TermRegion,
    sep1:        TermRegion,
    players_l:   TermRegion,
    log:         TermRegion,
    players_r:   TermRegion,
    sep2:        TermRegion,
    title2:      TermRegion,
    subtitle2:   TermRegion,
    sep3:        TermRegion,
    instruction: TermRegion,
    sep4:        TermRegion,
    input:       TermRegion,

    input_buffer: String,
    log_buffer: VecDeque<String>,
}

impl TermUi {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide, Clear(ClearType::All))?;

        let (width, height) = terminal::size()?;

        let mut tui = Self {
            width,
            height,

            title:       TermRegion::default(),
            subtitle:    TermRegion::default(),
            sep1:        TermRegion::default(),

            players_l:   TermRegion::default(),
            log:         TermRegion::default(),
            players_r:   TermRegion::default(),

            sep2:        TermRegion::default(),
            title2:      TermRegion::default(),
            subtitle2:   TermRegion::default(),
            sep3:        TermRegion::default(),
            instruction: TermRegion::default(),
            sep4:        TermRegion::default(),
            input:       TermRegion::default(),

            input_buffer: String::new(),
            log_buffer: VecDeque::new(),
        }; 
        tui.on_resize(width, height);

        Ok(tui)
    }

    fn compute_regions(&mut self) {
        // Fixed rows from top:
        //   0: Title
        //   1: SubTitle
        //   2: ────
        //   3..h-7: PlayerList | Log | PlayerList
        //   h-7: ────
        //   h-6: Title2
        //   h-5: SubTitle2
        //   h-4: ────
        //   h-3: Instruction
        //   h-2: ────
        //   h-1: Input

        let width = self.width;
        let height = self.height;

        let sep_row = |r: u16| TermRegion { row: r, col: 0, width, height: 1 };

        
        self.title       = TermRegion { row: 0, col: 0, width, height: 1 };
        self.subtitle    = TermRegion { row: 1, col: 0, width, height: 1 };
        self.sep1        = sep_row(2);

        let mid_top = 3;
        let mid_bottom = height - 7;
        let mid_height = mid_bottom.saturating_sub(mid_top);

        let content_width = width;

        let player_width = content_width * 3 / 10;
        let log_width = content_width - player_width * 2;

        self.players_l = TermRegion {
            row: mid_top,
            col: 0,
            width: player_width,
            height: mid_height,
        };

        self.log = TermRegion {
            row: mid_top,
            col: self.players_l.col + self.players_l.width,
            width: log_width,
            height: mid_height,
        };

        self.players_r = TermRegion {
            row: mid_top,
            col: self.log.col + self.log.width,
            width: player_width,
            height: mid_height,
        };

        self.sep2        = sep_row(mid_bottom);
        self.title2      = TermRegion { row: mid_bottom + 1, col: 0, width, height: 1 };
        self.subtitle2   = TermRegion { row: mid_bottom + 2, col: 0, width, height: 1 };
        self.sep3        = sep_row(mid_bottom + 3);
        self.instruction = TermRegion { row: mid_bottom + 4, col: 0, width, height: 1 };
        self.sep4        = sep_row(mid_bottom + 5);
        self.input       = TermRegion { row: height - 1, col: 0, width, height: 1 };
    }

    pub fn set_top_title(&mut self, text: &str) -> io::Result<()> {
        self.write_region_line(&self.title, 0, text)
    }

    pub fn set_top_subtitle(&mut self, text: &str) -> io::Result<()> {
        self.write_region_line(&self.subtitle, 0, text)
    }

    pub fn set_bot_title(&mut self, text: &str) -> io::Result<()> {
        self.write_region_line(&self.title2, 0, text)
    }

    pub fn set_bot_subtitle(&mut self, text: &str) -> io::Result<()> {
        self.write_region_line(&self.subtitle2, 0, text)
    }

    pub fn set_instruction(&mut self, text: &str) -> io::Result<()> {
        self.write_region_line(&self.instruction, 0, text)
    }
    // ─── Generic region writer ───────────────────────────────────


    fn fit_to_width(text: &str, width: usize) -> String {
        let mut result = String::new();
        let mut current_width = 0;

        for c in text.chars() {
            let char_width = UnicodeWidthChar::width(c).unwrap_or(0);

            if current_width + char_width > width {
                break;
            }

            result.push(c);
            current_width += char_width;
        }

        result
    }
    /// Write a line within a multi-line region (row offset from region top).
    fn write_region_line(
        &self,
        region: &TermRegion,
        offset: u16,
        text: &str,
    ) -> io::Result<()> {
        let row = region.row + offset;
        let width = region.width as usize;

        let text = Self::fit_to_width(text, width);
        let padding = width.saturating_sub(text.width());

        let line = format!("{}{}", text, " ".repeat(padding));

        execute!(
            io::stdout(),
            cursor::MoveTo(region.col, row),
            style::Print(line),
        )
    }

    // ─── Separators ──────────────────────────────────────────────

    fn draw_separator(&self, region: &TermRegion) -> io::Result<()> {
        let line = "─".repeat(region.width as usize);
        execute!(
            io::stdout(),
            cursor::MoveTo(region.col, region.row),
            Clear(ClearType::CurrentLine),
            style::Print(&line),
        )
    }

    fn draw_all_separators(&self) -> io::Result<()> {
        self.draw_separator(&self.sep1)?;
        self.draw_separator(&self.sep2)?;
        self.draw_separator(&self.sep3)?;
        self.draw_separator(&self.sep4)?;
        Ok(())
    }

    // ─── Input ───────────────────────────────────────────────────

    fn draw_input_line(&self) -> io::Result<()> {
        let full = format!("> {}█", self.input_buffer);
        execute!(
            io::stdout(),
            cursor::MoveTo(self.input.col, self.input.row),
            Clear(ClearType::CurrentLine),
            style::Print(&full),
        )
    }

    fn handle_key(&mut self, key: &crossterm::event::KeyEvent) -> Option<KeyResult> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && key.code == KeyCode::Char('c') {
                return Some(KeyResult::Exit);
        }
        match key.code {
            KeyCode::Char(c) => { self.input_buffer.push(c); return None; },
            KeyCode::Backspace => { self.input_buffer.pop(); return None; }
            KeyCode::Enter => {
                if self.input_buffer.is_empty() { return None; }
                let submitted = std::mem::take(&mut self.input_buffer);
                return Some(KeyResult::Submitted(submitted));
            }
            _ => {}
        }
        None
    }

    pub fn poll_key(&mut self) -> io::Result<Option<KeyResult>> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => {
                    let result = self.handle_key(&event);
                    self.draw_input_line()?;
                    return Ok(result);
                }
                Event::Mouse(_event) => (),
                Event::Paste(_data) => (),
                Event::Resize(width, height) => self.on_resize(width, height),
            }
        }
        Ok(None)
    }
    // ─── Log (scrolling within its region) ───────────────────────

    pub fn push_log(&mut self, text: &str) -> io::Result<()> {
        let cap = self.log.height as usize;
        self.log_buffer.push_back(text.to_string());
        while self.log_buffer.len() > cap {
            self.log_buffer.pop_front();
        }
        self.render_log()?;
        Ok(())
    }

    pub fn push_logs(&mut self, messages: Vec<String>) -> io::Result<()> {
        let capacity = self.log.height as usize;

        for message in messages {
            self.log_buffer.push_back(message);

            while self.log_buffer.len() > capacity {
                self.log_buffer.pop_front();
            }
        }

        self.render_log()?;
        Ok(())
    }

    
    fn render_log(&self) -> io::Result<()> {
        let lines = self.log_buffer.len();
        let start = (self.log.height as usize).saturating_sub(lines);

        let inner_width = (self.log.width as usize).saturating_sub(2);

        for i in 0..self.log.height as usize {
            let content = if i >= start {
                self.log_buffer
                    .get(i - start)
                    .map(String::as_str)
                    .unwrap_or("")
            } else {
                ""
            };

            let text = Self::fit_to_width(content, inner_width);

            let padding = inner_width.saturating_sub(text.width());

            let line = format!(
                "│{}{}│",
                text,
                " ".repeat(padding),
            );

            self.write_region_line(&self.log, i as u16, &line)?;
        }

        Ok(())
    }
    

    pub fn set_players_l(
        &self,
        title: &str, 
        player_infos: &[PlayerDisplay]
    ) -> io::Result<()> {
        self.render_players(&self.players_l, title, player_infos)
    }

    pub fn set_players_r(
        &self,
        title: &str, 
        player_infos: &[PlayerDisplay]
    ) -> io::Result<()> {
        self.render_players(&self.players_r, title, player_infos)
    }

    fn render_players(
        &self,
        region: &TermRegion,
        title: &str, 
        player_infos: &[PlayerDisplay]
    ) -> io::Result<()> {
        // Clear the entire region first.
        for row in 0..region.height {
            self.write_region_line(region, row, "")?;
        }

        // Title
        const TITLE_HEIGHT: usize = 2;
        self.write_region_line(region, 0, title)?;
        self.write_region_line(region, 1, "")?;

        // Each player needs:
        //   1 row -> name
        //   1 row -> separator (except the last visible player)
        const PLAYER_HEIGHT: usize = 1;

        let max_players = (region.height as usize - TITLE_HEIGHT) / PLAYER_HEIGHT;
        let visible_players = player_infos.iter().take(max_players);
               
        for (index, player) in visible_players.enumerate() {
            let row = TITLE_HEIGHT + (index * PLAYER_HEIGHT);

            // Player name-status
            self.write_region_line(
                region,
                row as u16,
                &format!("{} {}",player.name,player.status),
            )?;
            
        }

        Ok(())
    }

    // ─── Resize ──────────────────────────────────────────────────

    fn on_resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.compute_regions();

        // Trim log buffer to new capacity
        let cap = self.log.height as usize;
        while self.log_buffer.len() > cap {
            self.log_buffer.pop_front();
        }

        // Re-draw everything
        let _ = self.draw_all_separators();
        let _ = self.render_log();
        let _ = self.draw_input_line();
    }

    pub fn restore(&self) -> io::Result<()> {
        execute!(io::stdout(), cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
