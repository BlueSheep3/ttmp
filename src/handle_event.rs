use crate::{Message, Model};
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

pub fn handle_event(_model: &Model, event: Event) -> Option<Message> {
	match event {
		Event::Key(key) => {
			if key.kind == KeyEventKind::Release {
				return None;
			}
			if !key.modifiers.contains(KeyModifiers::CONTROL) {
				match key.code {
					KeyCode::Char(c) => return Some(Message::TypedChar(c)),
					KeyCode::Backspace => return Some(Message::Backspace),
					KeyCode::Enter => return Some(Message::Enter),
					_ => (),
				}
			}
			match key.code {
				KeyCode::Char('q') => Some(Message::Quit { save: true }),
				KeyCode::Char('e') => Some(Message::Error),
				KeyCode::Char('p') => Some(Message::Panic),
				_ => None,
			}
		}
		_ => None,
	}
}
