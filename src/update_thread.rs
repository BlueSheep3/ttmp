use crate::{
	command::match_input,
	config::{load as load_config, Config},
	input_thread::INPUT_Y,
};
use crossterm::{
	cursor::{MoveTo, RestorePosition, SavePosition},
	execute,
	terminal::{Clear, ClearType},
};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
	fs::File,
	io::{stdout, BufReader},
	path::PathBuf,
	sync::mpsc::Receiver,
	thread::sleep,
	time::{Duration, Instant},
};

const SLEEP_TIME: u64 = 250;

// Function to update and render changing information in a separate thread
pub fn main(receiver: &Receiver<String>) {
	// load config
	let mut config = load_config().expect("config should be valid RON");

	// setup sink
	let (_stream, stream_handle) = OutputStream::try_default().unwrap();
	let sink = Sink::try_new(&stream_handle).unwrap();

	// load in first song
	let mut last_loop_time = Instant::now();
	let mut current_song_name = String::new();
	let mut current_song = PathBuf::new();

	if !config.remaining.is_empty() {
		load_first_song(&config, &sink, &mut current_song_name, &mut current_song);
	} else {
		current_song_name = "[No Songs Remaining]".to_string();
	}

	// starting sink values
	sink.pause();
	sink.set_speed(config.speed);
	sink.set_volume(config.volume);

	execute!(stdout(), SavePosition).unwrap();
	print_song_info(&current_song_name, &config);
	execute!(stdout(), RestorePosition).unwrap();

	// Update and render loop
	loop {
		execute!(stdout(), SavePosition).unwrap();

		// Receive user input (if any)
		if let Ok(input) = receiver.try_recv() {
			execute!(
				stdout(),
				MoveTo(0, INPUT_Y + 2),
				Clear(ClearType::FromCursorDown)
			)
			.unwrap();

			if match_input(&input, &sink, &mut config) {
				if input == "q!" {
					// stop program without saving
					return;
				}
				break;
			}

			if config.remaining.is_empty() {
				sink.pause();
				current_song_name = "[No Songs Remaining]".to_string();
			}

			print_song_info(&current_song_name, &config);
		}

		// update progress time
		if !sink.is_paused() {
			config.current_progress += last_loop_time.elapsed();
		}

		// go to the next song if the current one is finished
		if sink.empty() && !config.remaining.is_empty() {
			if current_song == config.remaining[0] {
				config.remaining.remove(0);
			}

			config.current_progress = Duration::ZERO;
			if !config.remaining.is_empty() {
				load_first_song(&config, &sink, &mut current_song_name, &mut current_song);
			} else {
				sink.pause();
				current_song_name = "[No Songs Remaining]".to_string();
			}
			print_song_info(&current_song_name, &config);
		}

		// move the cursor back to allow for user input to not be glitchy
		execute!(stdout(), RestorePosition).unwrap();

		last_loop_time = Instant::now();

		// Adjust the sleep duration based on how often you want to update the display
		sleep(Duration::from_millis(SLEEP_TIME));
	}

	config.save().unwrap();
}

fn print_song_info(current_song_name: &String, config: &Config) {
	execute!(stdout(), MoveTo(0, 0), Clear(ClearType::CurrentLine)).unwrap();
	println!("Music: {}", current_song_name);
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
	println!("Songs Remaining: {}", config.remaining.len());
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
}

fn load_first_song(config: &Config, sink: &Sink, song_name: &mut String, song: &mut PathBuf) {
	let path = config.parent_path.join(&config.remaining[0]);
	let file = File::open(path).unwrap();
	let source = Decoder::new(BufReader::new(file))
		.unwrap()
		.skip_duration(config.current_progress);
	sink.append(source);

	*song = config.remaining[0].clone();

	*song_name = config.remaining[0]
		.file_name()
		.unwrap()
		.to_string_lossy()
		.to_string();
}
