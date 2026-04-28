// src/input.rs

use crossterm::event::{read, Event, KeyCode, KeyEventKind, poll};
use crate::buffer::EditorBuffer;
use std::collections::HashSet;
use std::io::Result;
use std::time::Duration;

#[derive(PartialEq)]
pub enum InputMode {
		Normal,
		Insert,
    Finding,
    EnteringFileNameOpen,
    EnteringFileNameSave,
}

#[derive(Debug)]
pub enum Command {
    Quit,
    InsertChar(char),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Backspace,
    InsertNewline,
    Undo,
    Redo,
    StartFind,
    ConfirmFind,
    StartOpenFile,
    ConfirmOpenFile,
    StartSaveFile,
    ConfirmSaveFile,
}

pub struct InputHandler {
    pub mode: InputMode,
    pub filename_input: String,
    pub find_input: String,
    pub confirmed_find_term: Option<String>,
		pub status_message: Option<String>,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            mode: InputMode::Normal,
            filename_input: String::new(),
            find_input: String::new(),
            confirmed_find_term: None,
						status_message: None, 
        }
    }

    pub fn get_mode(&self) -> &InputMode {
        &self.mode
    }

    /// Reads and parses command input from terminal.
    /// Returns Some(Command) if an actionable command is parsed.
    pub fn process_input(&mut self) -> Result<Option<Command>> {
				if !poll(Duration::from_millis(0))? {
					return Ok(None);
				}

				let event = read()?;

				let key_event = match event {
					Event::Key(k) if k.kind == KeyEventKind::Press => k,
          _ => return Ok(None),
        };
            match self.mode {
								InputMode::Normal => {
									match key_event.code {
										KeyCode::Char('i') => {
											self.status_message = None;
											self.mode = InputMode::Insert;
											return Ok(None);
										}
										
										KeyCode::Char('f') => {
											self.confirmed_find_term = None;
											self.start_find();

											while poll(Duration::from_millis(0))? {
												let _ = read();
											}

											return Ok(None);
										}

										KeyCode::Char('o') => {
											self.start_open_file();
						
											while poll(Duration::from_millis(0))? {
												let _ = read();
											}

											return Ok(None);
										}
			
										KeyCode::Char('s') => {
											self.start_save_file();
											
											while poll(Duration::from_millis(0))? {
												let _ = read();
											}

											return Ok(None);
										}

										KeyCode::Char('h') => return Ok(Some(Command::MoveLeft)), 
										KeyCode::Char('l') => return Ok(Some(Command::MoveRight)),
										KeyCode::Char('k') => return Ok(Some(Command::MoveUp)),
										KeyCode::Char('j') => return Ok(Some(Command::MoveDown)),
										KeyCode::Char('q') => return Ok(Some(Command::Quit)),	
                    _ => return Ok(None),
                  }
               	}
								
								InputMode::Insert => {
									match key_event.code {
										KeyCode::Esc => {
											self.mode = InputMode::Normal;
											return Ok(None);
										}
										KeyCode::Char(c) => { return Ok(Some(Command::InsertChar(c))); }
                    KeyCode::Backspace => return Ok(Some(Command::Backspace)),
                    KeyCode::Enter => return Ok(Some(Command::InsertNewline)),
                    _ => return Ok(None), 
               		}
								}
                InputMode::Finding => {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.mode = InputMode::Normal;
														self.find_input.clear();
                            return Ok(None);
                        }

                        KeyCode::Enter => {
                            return Ok(Some(Command::ConfirmFind));
                        }

                        KeyCode::Backspace => {
                            self.find_input.pop();
                            return Ok(None);
                        }

                        KeyCode::Char(c) => {
                            self.find_input.push(c);
                            return Ok(None);
                        }

                        _ => return Ok(None),
                    }
                }
                InputMode::EnteringFileNameOpen => match key_event.code {
                    KeyCode::Esc => {
                        self.mode = InputMode::Normal;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        self.mode = InputMode::Normal;
                        return Ok(Some(Command::ConfirmOpenFile));
                    }
                    KeyCode::Backspace => {
                        self.filename_input.pop();
                        return Ok(None);
                    }
                    KeyCode::Char(c) => {
                        self.filename_input.push(c);
                        return Ok(None);
                    }
                    _ => return Ok(None),
                },

                InputMode::EnteringFileNameSave => match key_event.code {
                    KeyCode::Esc => {
                        self.mode = InputMode::Normal;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        self.mode = InputMode::Normal;
                        return Ok(Some(Command::ConfirmSaveFile));
                    }
                    KeyCode::Backspace => {
                        self.filename_input.pop();
                        return Ok(None);
                    }
                    KeyCode::Char(c) => {
                        self.filename_input.push(c);
                        return Ok(None);
                    }
                    _ => return Ok(None), 
                },
            }

        Ok(None)
    }

    pub fn start_find(&mut self) {
				self.status_message = None;
        self.mode = InputMode::Finding;
        self.find_input.clear();
    }

    pub fn start_open_file(&mut self) {
				self.status_message = None;
        self.mode = InputMode::EnteringFileNameOpen;
        self.filename_input.clear();
    }

    pub fn start_save_file(&mut self) {
				self.status_message = None;
        self.mode = InputMode::EnteringFileNameSave;
        self.filename_input.clear();
    }

    pub fn confirm_find(&mut self, buffer: &EditorBuffer, dirty_lines: &mut std::collections::HashSet<usize>) {
				let term = self.find_input.clone();

        if term.is_empty() {
            self.mode  = InputMode::Normal;
						return;
				}
        self.confirmed_find_term = Some(term.clone());
				
				let mut found = false;

				for i in 0..buffer.len_lines() {
					let line = buffer.line(i).to_string();

					if line.contains(&term) {
						dirty_lines.insert(i);
						found = true;
					}
				}
				self.mode = InputMode::Normal;

				if !found {
					self.status_message = Some("-- Not Found --".to_string());
				} else {
					self.status_message = None;
				}
    }

    pub fn confirm_open_file(&mut self) -> Option<String> {
        if self.filename_input.is_empty() {
            self.status_message = Some("-- File Not Found --".to_string());
						None
        } else {
						self.status_message = Some("-- Opened File --".to_string());
            Some(self.filename_input.clone())
        }
    }

    pub fn confirm_save_file(&mut self) -> Option<String> {
        if self.filename_input.is_empty() {
            self.status_message = Some("-- Cannot Save File --".to_string());
						None
        } else {
						self.status_message = Some("-- Save File --".to_string());
            Some(self.filename_input.clone())
        }
    }
}
