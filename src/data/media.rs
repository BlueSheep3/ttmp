use super::{context::Context, error::Result, playlist};
use souvlaki::{
	MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
	SeekDirection,
};
use std::{
	borrow::Cow::{Borrowed, Owned},
	sync::mpsc::Sender,
};

pub struct Media {
	pub controls: MediaControls,
	#[cfg(target_os = "windows")]
	pub dummy_window: self::windows::DummyWindow,
}

impl Context {
	// You must call the specific media update functions whenever some
	// change to the corresponding data happens.
	// On windows, you must also call `windows::pump_event_queue()` repeatedly.

	// this REALLY doesn't like it if you send too many metadata updates too quickly, so use
	// the smaller functions when possible and space out any calls you need to do repeatedly.
	pub fn update_media_all(&mut self) -> Result<()> {
		self.update_media_metadata()?;
		self.update_media_volume()?;
		self.update_media_progress()?;
		Ok(())
	}

	pub fn update_media_metadata(&mut self) -> Result<()> {
		let song_name = self
			.playlist
			.remaining
			.front()
			.map(|f| playlist::get_song_name(f));
		let duration = self.get_current_duration();

		if let Some(media) = &mut self.media {
			media.controls.set_metadata(MediaMetadata {
				title: song_name.as_deref(),
				duration,
				..Default::default()
			})?;
		}
		Ok(())
	}

	pub fn update_media_volume(&mut self) -> Result<()> {
		// set_volume is onlz available on mpris
		#[cfg(all(
			unix,
			not(any(target_os = "macos", target_os = "ios", target_os = "android"))
		))]
		if let Some(media) = &mut self.media {
			media.controls.set_volume(self.config.volume as f64)?;
		}
		Ok(())
	}

	pub fn update_media_progress(&mut self) -> Result<()> {
		let Some(media) = &mut self.media else {
			return Ok(());
		};
		let progress = self.playlist.progress;
		let playback = if self.playlist.remaining.is_empty() {
			MediaPlayback::Stopped
		} else if self.player.is_paused() {
			MediaPlayback::Paused {
				progress: Some(MediaPosition(progress)),
			}
		} else {
			MediaPlayback::Playing {
				progress: Some(MediaPosition(progress)),
			}
		};
		media.controls.set_playback(playback)?;
		Ok(())
	}
}

pub fn setup_media(cmd_sender: Sender<String>) -> Result<Media> {
	#[cfg(not(target_os = "windows"))]
	let hwnd = None;
	#[cfg(target_os = "windows")]
	let (hwnd, dummy_window) = {
		let dummy_window = windows::DummyWindow::new().unwrap();
		let handle = Some(dummy_window.handle.0 as _);
		(handle, dummy_window)
	};

	let config = PlatformConfig {
		dbus_name: "music.player.ttmp",
		display_name: "ttmp",
		hwnd,
	};

	let mut controls = MediaControls::new(config)?;
	controls.attach(move |event| {
		let cmd = match event {
			MediaControlEvent::Play => Borrowed("p+"),
			MediaControlEvent::Pause => Borrowed("p-"),
			MediaControlEvent::Toggle => Borrowed("p"),
			MediaControlEvent::Next => Borrowed("pn"),
			MediaControlEvent::Previous => Borrowed("pp"),
			MediaControlEvent::Stop => Borrowed("p-"),
			MediaControlEvent::Seek(dir) => Borrowed(match dir {
				SeekDirection::Forward => "gf 5s",
				SeekDirection::Backward => "gb 5s",
			}),
			MediaControlEvent::SeekBy(dir, amount) => Owned(match dir {
				SeekDirection::Forward => format!("gf {}s", amount.as_secs_f32()),
				SeekDirection::Backward => format!("gb {}s", amount.as_secs_f32()),
			}),
			MediaControlEvent::SetPosition(pos) => Owned(format!("g {}s", pos.0.as_secs_f32())),
			MediaControlEvent::SetVolume(vol) => Owned(format!("pv {vol}")),
			MediaControlEvent::OpenUri(_uri) => {
				eprintln!("the OpenUri media control is currently not supported"); // TODO
				return;
			}
			MediaControlEvent::Raise => return, // ttmp doesn't control the terminal window
			MediaControlEvent::Quit => Borrowed("q"),
		};
		if let Err(e) = cmd_sender.send(cmd.into_owned()) {
			eprintln!("error while sending command from media control event: {e}");
		}
	})?;

	let media = Media {
		controls,
		#[cfg(target_os = "windows")]
		dummy_window,
	};
	Ok(media)
}

// ########################## windows specific boilerplate ###############################

// this code is copied from:
// https://github.com/Sinono3/souvlaki/blob/023e1a98f2b31704cfc48c160809ed3c5e139345/examples/print_events.rs

#[cfg(target_os = "windows")]
#[allow(
	clippy::multiple_unsafe_ops_per_block,
	clippy::undocumented_unsafe_blocks
)]
mod windows {
	use std::{io::Error, mem};
	use windows::{
		Win32::{
			Foundation::{HWND, LPARAM, LRESULT, WPARAM},
			System::LibraryLoader::GetModuleHandleW,
			UI::WindowsAndMessaging::{
				CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GA_ROOT,
				GetAncestor, IsDialogMessageW, MSG, PM_REMOVE, PeekMessageW, RegisterClassExW,
				TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
			},
		},
		core::PCWSTR,
		w,
	};

	pub struct DummyWindow {
		pub handle: HWND,
	}

	impl DummyWindow {
		pub fn new() -> Result<DummyWindow, String> {
			let class_name = w!("SimpleTray");

			let handle_result = unsafe {
				let instance = GetModuleHandleW(None)
					.map_err(|e| format!("Getting module handle failed: {e}"))?;

				let wnd_class = WNDCLASSEXW {
					cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
					hInstance: instance,
					lpszClassName: PCWSTR::from(class_name),
					lpfnWndProc: Some(Self::wnd_proc),
					..Default::default()
				};

				if RegisterClassExW(&wnd_class) == 0 {
					return Err(format!(
						"Registering class failed: {}",
						Error::last_os_error()
					));
				}

				let handle = CreateWindowExW(
					WINDOW_EX_STYLE::default(),
					class_name,
					w!(""),
					WINDOW_STYLE::default(),
					0,
					0,
					0,
					0,
					None,
					None,
					instance,
					None,
				);

				if handle.0 == 0 {
					Err(format!(
						"Message only window creation failed: {}",
						Error::last_os_error()
					))
				} else {
					Ok(handle)
				}
			};

			handle_result.map(|handle| DummyWindow { handle })
		}
		extern "system" fn wnd_proc(
			hwnd: HWND,
			msg: u32,
			wparam: WPARAM,
			lparam: LPARAM,
		) -> LRESULT {
			unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
		}
	}

	impl Drop for DummyWindow {
		fn drop(&mut self) {
			unsafe {
				DestroyWindow(self.handle);
			}
		}
	}

	pub fn pump_event_queue() -> bool {
		unsafe {
			let mut msg: MSG = std::mem::zeroed();
			let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
			while msg.message != WM_QUIT && has_message {
				if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
					TranslateMessage(&msg);
					DispatchMessageW(&msg);
				}

				has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
			}

			msg.message == WM_QUIT
		}
	}
}
