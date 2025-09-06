use std::{sync::mpsc::Sender, thread, time::Duration};

#[cfg(target_os = "windows")]
pub fn main(sender: &Sender<String>) -> ! {
	use winapi::um::winuser::{
		GetAsyncKeyState as get_async_key_state, VK_MEDIA_PLAY_PAUSE, VK_VOLUME_MUTE,
	};

	let mut pause_pressed = false;
	loop {
		// SAFETY: The Windows API specifies that the input must be a virtual key,
		// which it is in this case, since we use the virtual key constants.
		// It also specifies that the most significant bit represents whether the key is down.
		// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate
		let play_state = unsafe { get_async_key_state(VK_MEDIA_PLAY_PAUSE) } as u16 & 0x8000 != 0;
		// SAFETY: see comment above
		let mute_state = unsafe { get_async_key_state(VK_VOLUME_MUTE) } as u16 & 0x8000 != 0;

		if play_state || mute_state {
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

// replacement function for compiling to linux
#[cfg(target_os = "linux")]
pub fn main(_sender: &Sender<String>) -> ! {
    // theres probably some way to do global hotkeys for linux,
    // but i just want it to compile at all first.
	loop {
		thread::sleep(Duration::from_secs(1 << 30));
	}
}
