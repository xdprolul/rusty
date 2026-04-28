// src/render.rs

use crate::buffer::EditorBuffer;
use crate::input::InputMode;
use crossterm::{
    cursor,
    style::{Print, Stylize},
    queue, ExecutableCommand,
};
use std::collections::HashSet;
use std::io::{Error, Stdout, Write};

pub struct Renderer {
    pub max_lines: usize,
    virtual_screen: VirtualScreen,
}

pub struct VirtualScreen {
    lines: Vec<String>,
}

impl VirtualScreen {
    pub fn new(rows: usize) -> Self {
        VirtualScreen {
            lines: vec!["".to_string(); rows],
        }
    }
    pub fn update_line(&mut self, index: usize, content: &str) {
        if index < self.lines.len() {
            self.lines[index] = content.to_string();
        }
    }
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }
}

impl Renderer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            virtual_screen: VirtualScreen::new(max_lines),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        stdout: &mut Stdout,
        buffer: &EditorBuffer,
        dirty_lines: &HashSet<usize>,
        viewport_row: usize,
        max_lines: usize,
        cursor_col: usize,
        current_line: usize,
        cursor_visible: bool,
        mode: &InputMode,
        filename_input: &str,
        find_input: &str,
        confirmed_find_term: &Option<String>,
				status_message: &Option<String>,
    ) -> Result<(), Error> {        
        let total_lines = buffer.len_lines();

        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
        write!(stdout, "Welcome to rusty")?;

        // Draw prompt/status line at bottom based on mode
        queue!(stdout, cursor::MoveTo(0, (max_lines + 1) as u16))?;
        queue!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
				if let Some(message) = status_message {
					write!(stdout, "{}", message)?;
				} else {
        	match mode {
            InputMode::EnteringFileNameOpen => {
                write!(stdout, "Open file: {}", filename_input)?;
            }
            InputMode::EnteringFileNameSave => {
                write!(stdout, "Save file: {}", filename_input)?;
            }
            InputMode::Finding => {
                write!(stdout, "Find: {}", find_input)?;
            }
            InputMode::Normal => {
            	write!(stdout, "-- NORMAL --")?; 
            }
						InputMode::Insert => {
							write!(stdout, "-- INSERT --")?;
						}
        	}
				}

        // Go through dirty lines and redraw
        for &line_idx in dirty_lines.iter() {
            if line_idx < viewport_row || line_idx >= viewport_row + max_lines {
                continue;
            }
            let view_line_idx = line_idx - viewport_row;

            if line_idx >= total_lines {
                // Draw "~" for empty lines outside buffer
                let tilde_line = format!("{:>width$}~ ", "", width = 3);
                let cached_line = self.virtual_screen.get_line(view_line_idx).unwrap_or("");
                if cached_line != tilde_line {
                    queue!(stdout, cursor::MoveTo(0, (view_line_idx + 1) as u16))?;
                    queue!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
                    write!(stdout, "{}", tilde_line)?;
                    self.virtual_screen.update_line(view_line_idx, &tilde_line);
                }
                continue;
            }

            let rope_line = buffer.line(line_idx);
            let line_str = if rope_line.len_chars() > 0 && rope_line.char(rope_line.len_chars() - 1) == '\n' {
                rope_line.slice(0..rope_line.len_chars() - 1).to_string()
            } else {
                rope_line.to_string()
            };

            let gutter_width = 4;
            let gutter = format!("{:>width$} ", line_idx + 1, width = gutter_width);

						let new_line = format!("{}{}", gutter, line_str);

						let cached_line = self.virtual_screen.get_line(view_line_idx).unwrap_or("");

						if line_idx != current_line && cached_line == new_line {
							continue;
						}

            queue!(stdout, cursor::MoveTo(0, (view_line_idx + 1) as u16))?;
            queue!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
            queue!(stdout, Print(&gutter))?;

            if let Some( find_term) = confirmed_find_term.as_ref() {
                let mut remaining = line_str.as_str();
                while !remaining.is_empty() {
                    if remaining.starts_with(find_term) {
                        queue!(stdout, Print(find_term.as_str().reverse()))?;
                        remaining = &remaining[find_term.len()..];
                    } else {
                        let ch = remaining.chars().next().unwrap();
                        let ch_len = ch.len_utf8();
                        queue!(stdout, Print(ch))?;
                        remaining = &remaining[ch_len..];
                    }
                }
            } else {
                queue!(stdout, Print(&line_str))?;
            }
						
            self.virtual_screen.update_line(view_line_idx, &new_line);
        }

        // Draw cursor position
        let cursor_y = (current_line.saturating_sub(viewport_row) + 1) as u16;

				let rope_line = buffer.line(current_line);

				let cursor_line_str = if rope_line.len_chars() > 0 && rope_line.char(rope_line.len_chars() - 1) == '\n' {
					rope_line.slice(0..rope_line.len_chars() - 1).to_string()
				} else {
					rope_line.to_string()
				};
	
				let mut visual_x = 0;
				let mut cursor_screen_x = 0;
				
				for (i, ch) in cursor_line_str.chars().enumerate() {
					if i == cursor_col {
						cursor_screen_x = visual_x;
					}
					visual_x += 1;
				}

				if cursor_col >= cursor_line_str.chars().count() {
					cursor_screen_x = visual_x;
				}

				let gutter_width = 4;
        let cursor_x = (gutter_width + 1 + cursor_screen_x) as u16;
        queue!(stdout, cursor::MoveTo(cursor_x, cursor_y))?;

        if cursor_visible {
            queue!(stdout, cursor::Show)?;
        } else {
            queue!(stdout, cursor::Hide)?;
        }

        stdout.flush()?;
        Ok(())
    }
}
