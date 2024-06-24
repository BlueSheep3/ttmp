use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{
	GetAsyncKeyState as get_async_key_state, VK_MEDIA_PLAY_PAUSE, VK_VOLUME_MUTE,
};

pub fn main(sender: &Sender<String>) -> ! {
	let mut pause_pressed = false;
	loop {
		// SAFETY: accesses global keyboard state
		let play_pause_state =
			unsafe { get_async_key_state(VK_MEDIA_PLAY_PAUSE) } as u16 & 0x8000 != 0;
		let mute_state = unsafe { get_async_key_state(VK_VOLUME_MUTE) } as u16 & 0x8000 != 0;
		if play_pause_state || mute_state {
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
