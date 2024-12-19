mod mode;
use mode::Mode;
use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Direction, Layout, Position},
    symbols::border,
    text::Text,
    widgets::{Block, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};
use std::{fs, io};

pub struct App {
    file_name: String,
    contents: Vec<String>,

    mode: Mode,
    exit: bool,

    row: usize,
    col: usize,
    target_col: usize,

    view: (usize, usize),

    command: String,
}

impl App {
    pub fn run(terminal: &mut DefaultTerminal, file_name: String) -> io::Result<()> {
        let mut contents: Vec<String> = vec![];
        let file_contents = fs::read_to_string(&file_name);
        if let Ok(fcontents) = file_contents {
            fcontents
                .trim()
                .split('\n')
                .for_each(|x| contents.push(x.to_string()));
        } else {
            contents.push(String::new());
        }
        let contents_len = contents.len();
        let mut app = App {
            file_name,
            contents,
            command: String::new(),
            mode: Mode::Normal,
            exit: false,

            view: (0, contents_len),

            row: 0,
            col: 0,
            target_col: 0,
        };
        terminal.show_cursor()?;
        while !app.exit {
            terminal.draw(|frame| {
                app.draw(frame);
                frame.set_cursor_position(app.get_cur_pos(frame.area().height));
            })?;
            app.handle_input();
        }

        Ok(())
    }

    fn get_cur_pos(&mut self, height: u16) -> Position {
        match self.mode {
            Mode::Command => Position::new(self.command.len() as u16, height - 1),
            _ => Position::new(self.col as u16 + 1, self.row as u16 + 1),
        }
    }

    fn handle_input(&mut self) {
        if let Ok(event::Event::Key(key)) = event::read() {
            match self.mode {
                Mode::Insert => self.handle_keycode_insert(key.code),
                Mode::Normal => self.handle_keycode_normal(key.code),
                Mode::Command => self.handle_keycode_command(key.code),
            }
        }
    }

    fn handle_command(&mut self) {
        match self.command.trim() {
            ":q" => self.exit = true,
            ":w" => self.write_to_file(),
            ":wq" | ":qw" => {
                self.write_to_file();
                self.exit = true;
            }
            _ => self.command = String::from(format!("Unknown command: {}", &self.command[1..])),
        }
    }

    fn handle_keycode_insert(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(ch) => {
                self.handle_ch_insert(ch);
            }
            KeyCode::Backspace => {
                if self.col == 0 && self.row > 0 {
                    let from_right = self.contents[self.row - 1].len() - 1;
                    let next = self.contents[self.row].to_string();
                    self.contents[self.row - 1] += &next;
                    if self.row >= self.contents.len().saturating_sub(1) {
                        self.contents.pop();
                    } else {
                        self.contents.remove(self.row);
                    }
                    self.view.1 = self.view.1.saturating_sub(1);
                    self.row -= 1;
                    self.col = from_right + 1;
                } else {
                    if self.col >= self.contents[self.row].len().saturating_sub(1) {
                        self.contents[self.row].pop();
                    } else {
                        self.contents[self.row].remove(self.col.saturating_sub(1));
                    }
                    self.col = self.col.saturating_sub(1);
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                let len = self.contents[self.row].len().saturating_sub(1);
                if self.col > len {
                    self.col = len;
                }
                self.target_col = self.col;
            }
            KeyCode::Enter => {
                if self.row == self.contents.len().saturating_sub(1) {
                    self.contents.push(String::new());
                    self.row = self.contents.len() - 1;
                    self.col = 0;
                } else {
                    self.contents.insert(self.row + 1, String::new());
                    let remainder = self.contents[self.row][self.col..].to_string();
                    self.contents[self.row].replace_range(self.col.., "");
                    self.row += 1;
                    self.contents[self.row] = remainder;
                    self.col = 0;
                }
                self.view.1 += 1;
            }
            _ => (),
        }
    }
    fn handle_ch_insert(&mut self, ch: char) {
        self.contents[self.row].insert(self.col, ch);
        self.col += 1;
    }

    fn handle_keycode_normal(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(ch) => self.handle_ch_normal(ch),
            _ => (),
        }
    }
    fn handle_ch_normal(&mut self, ch: char) {
        match ch {
            ':' => {
                self.mode = Mode::Command;
                self.command = String::from(":");
            }
            'h' => self.move_left(),
            'l' => self.move_right(),
            'k' => self.move_up(),
            'j' => self.move_down(),
            'i' => {
                self.mode = Mode::Insert;
            }
            'I' => {
                self.mode = Mode::Insert;
                self.target_col = 0;
                self.col = 0;
            }
            'a' => {
                self.mode = Mode::Insert;
                if !self.contents[self.row].is_empty() {
                    self.col += 1;
                }
            }
            'A' => {
                self.mode = Mode::Insert;
                self.target_col = usize::MAX;
                self.col = self.contents[self.row].len();
            }
            '0' => {
                self.col = 0;
                self.target_col = 0;
            }
            '$' => {
                self.target_col = usize::MAX;
                self.col = self.contents[self.row].len() - 1
            }
            'G' => {
                self.row = self.contents.len() - 1;
                self.target_col = usize::MAX;
                self.col = self.contents[self.row].len() - 1;
            }
            'g' => {
                self.row = 0;
                self.col = 0;
                self.target_col = 0;
            }
            'x' => {
                if self.contents[self.row].is_empty() {
                    return;
                }
                if self.col >= self.contents[self.row].len().saturating_sub(1) {
                    self.contents[self.row].pop();
                } else {
                    self.contents[self.row].remove(self.col);
                    if self.col > self.contents[self.row].len().saturating_sub(1) {
                        self.col -= 1;
                    }
                }
            }
            'o' => {
                if self.row == self.contents.len().saturating_sub(1) {
                    self.contents.push(String::new());
                    self.row = self.contents.len() - 1;
                } else {
                    self.contents.insert(self.row + 1, String::new());
                    self.row += 1;
                }
                self.col = 0;
                self.mode = Mode::Insert;
                self.view.1 += 1;
            }
            'O' => {
                if self.contents.is_empty() {
                    self.contents.push(String::new());
                } else {
                    self.contents.insert(self.row, String::new());
                }
                self.col = 0;
                self.mode = Mode::Insert;
                self.view.1 += 1;
            }
            _ => (),
        }
    }

    fn handle_keycode_command(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(ch) => self.command += &ch.to_string(),
            KeyCode::Backspace => {
                if self.command.len() > 1 {
                    self.command.pop();
                }
            }
            KeyCode::Enter => {
                self.handle_command();
                self.mode = Mode::Normal
            }
            KeyCode::Esc => {
                self.command.clear();
                self.mode = Mode::Normal;
            }
            _ => (),
        };
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn write_to_file(&mut self) {
        if let Err(_) = fs::write(&self.file_name, &self.contents.join("\n")) {
            self.command = String::from("Could not write to file.");
        } else {
            let bytes = fs::metadata(&self.file_name).unwrap().len();
            self.command = format!("\"{}\", {}KB written", self.file_name, bytes)
        }
    }

    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            self.target_col = self.col;
        }
    }
    fn move_right(&mut self) {
        if self.col < self.contents[self.row].len() {
            self.col += 1;
            self.target_col = self.col;
        }
    }
    fn move_down(&mut self) {
        if self.row < self.contents.len().saturating_sub(1) {
            self.row += 1;
        }
        let len = self.contents[self.row].len().saturating_sub(1);
        if self.target_col > len {
            self.col = len;
        } else {
            self.col = self.target_col;
        }
    }
    fn move_up(&mut self) {
        self.row = self.row.saturating_sub(1);
        let len = self.contents[self.row].len().saturating_sub(1);
        if self.target_col > len {
            self.col = len;
        } else {
            self.col = self.target_col;
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let ver = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(0),          // Space above the widget
                Constraint::Percentage(100), // Height of the widget (centered area)
                Constraint::Length(1),       // Space below the widget
            ])
            .split(area);

        let editor_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(0),   // Space on the left
                Constraint::Percentage(100), // Centered widget area
                Constraint::Percentage(0),   // Space on the right
            ])
            .split(ver[1]);

        let command_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(0),
                Constraint::Percentage(100),
                Constraint::Percentage(0),
            ])
            .split(ver[2]);

        let paragraph = Paragraph::new(Text::from(
            self.contents[self.view.0..self.view.1].join("\n"),
        ))
        .wrap(Wrap::default());
        let block = Block::bordered()
            .border_set(border::DOUBLE)
            .title_top(&*self.file_name)
            .title_bottom(match self.mode {
                Mode::Normal => "═ NORMAL ",
                Mode::Insert => "═ INSERT ",
                Mode::Command => "═ COMMAND ",
            });
        paragraph.block(block).render(editor_area[1], buf);
        Text::from(&*self.command).render(command_area[1], buf);
    }
}
