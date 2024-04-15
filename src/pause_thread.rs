use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{GetAsyncKeyState as get_async_key_state, VK_MEDIA_PLAY_PAUSE};

pub fn main(sender: &Sender<String>) -> ! {
	let mut pause_pressed = false;
	loop {
		// SAFETY: VK_MEDIA_PLAY_PAUSE is a valid virtual key code
		if unsafe { get_async_key_state(VK_MEDIA_PLAY_PAUSE) } as u16 & 0x8000 != 0 {
			if !pause_pressed {
				pause_pressed = true;
				sender
					.send("p".to_owned())
					.expect("Failed to send pause input");
			}
		} else {
			pause_pressed = false;
		}
		thread::sleep(Duration::from_millis(50));
	}
}
