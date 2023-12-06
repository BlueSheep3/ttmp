use crate::readln;
use crossterm::{
	cursor::MoveTo,
	execute,
	terminal::{Clear, ClearType::UntilNewLine},
};
use std::{io::stdout, sync::mpsc::Sender};

pub const INPUT_Y: u16 = 5;

// Function to handle user input in a separate thread from rendering
pub fn main(sender: &Sender<String>) {
	loop {
		execute!(stdout(), MoveTo(0, INPUT_Y)).expect("couldn't move cursor to 0");
		let input = readln!("Command: ");

		sender.send(input.clone()).expect("Failed to send input");

		execute!(stdout(), MoveTo(9, INPUT_Y), Clear(UntilNewLine))
			.expect("couldn't move cursor to 9");
	}
}
