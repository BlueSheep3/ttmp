// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

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

fn handle_key_normal_mode(model: &Model, key: KeyEvent) -> Option<Message> {
	if key.modifiers.contains(KeyModifiers::CONTROL) {
		return None;
	}

	model
		.ctx
		.config
		.keybinds
		.iter()
		.find(|(k, _)| *k == key.code)
		.map(|(_, m)| m.clone())
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
