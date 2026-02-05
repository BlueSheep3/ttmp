use crate::{Message, Model};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn handle_event(model: &Model, event: Event) -> Option<Message> {
	match event {
		Event::Key(key) => {
			if key.kind == KeyEventKind::Release {
				return None;
			}

			if (key.modifiers.contains(KeyModifiers::CONTROL)
				&& matches!(key.code, KeyCode::Char('c')))
				|| matches!(key.code, KeyCode::Esc)
			{
				return Some(Message::GotoNormalMode);
			}
			if key.modifiers.contains(KeyModifiers::CONTROL)
				&& matches!(key.code, KeyCode::Char('r'))
			{
				return Some(Message::ToggleScreenRedraws);
			}

			if model.current_command.is_some() {
				handle_key_command_mode(model, key)
			} else {
				handle_key_normal_mode(model, key)
			}
		}
		_ => None,
	}
}

fn handle_key_normal_mode(_model: &Model, key: KeyEvent) -> Option<Message> {
	if key.modifiers.contains(KeyModifiers::CONTROL) {
		return None;
	}

	match key.code {
		KeyCode::Char(':' | ';' | 'c') => Some(Message::GotoCommandMode),
		KeyCode::Char('q') => Some(Message::Quit { save: true }),
		KeyCode::Char('?') => Some(Message::RunCommand("h")),
		KeyCode::Char('S') => Some(Message::RunCommand("s")),

		KeyCode::Char(' ') => Some(Message::RunCommand("p")),
		KeyCode::Char('p') => Some(Message::RunCommand("p-")),
		KeyCode::Char('P') => Some(Message::RunCommand("p+")),
		KeyCode::Right => Some(Message::RunCommand("gf 5s")),
		KeyCode::Left => Some(Message::RunCommand("gb 5s")),
		KeyCode::Up => Some(Message::RunCommand("pv+ 5")),
		KeyCode::Down => Some(Message::RunCommand("pv- 5")),
		KeyCode::Char('0') => Some(Message::RunCommand("g")),

		KeyCode::Char('r') => Some(Message::RunCommand("r")),
		KeyCode::Char('j') => Some(Message::RunCommand("pn")),

		KeyCode::Char('f') => Some(Message::StartCommand("fte ")),
		KeyCode::Char('F') => Some(Message::StartCommand("fta ")),
		KeyCode::Char('s') => Some(Message::StartCommand("fs ")),
		KeyCode::Char('t') => Some(Message::StartCommand("tac ")),
		KeyCode::Char('T') => Some(Message::StartCommand("trc ")),

		KeyCode::Char('l') => Some(Message::StartCommand("ls ")),
		KeyCode::Char('L') => Some(Message::RunCommand("lg")),

		KeyCode::Char('m') => Some(Message::RunCommand("ml")),
		KeyCode::Char('M') => Some(Message::StartCommand("ma ")),
		_ => None,
	}
}

fn handle_key_command_mode(_model: &Model, key: KeyEvent) -> Option<Message> {
	if key.modifiers.contains(KeyModifiers::CONTROL) {
		return None;
	}

	match key.code {
		KeyCode::Char(c) => Some(Message::TypedChar(c)),
		KeyCode::Backspace => Some(Message::Backspace),
		KeyCode::Enter => Some(Message::Enter),
		_ => None,
	}
}
