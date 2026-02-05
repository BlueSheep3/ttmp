use crate::{
	Model,
	data::context::ProgramMode,
	duration::{display_duration, display_duration_out_of},
};
use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout, Rect},
	style::Style,
	text::{Line, Span, Text},
	widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn view(model: &Model, frame: &mut Frame) {
	if model.ctx.config.dont_redraw_screen {
		dont_draw_screen_replacement(frame, frame.area());
		return;
	}

	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Fill(1), Constraint::Length(4)])
		.split(frame.area());

	if model.current_command.is_some() || !model.ctx.cmd_out.is_empty() {
		let layout = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
			.split(layout[0]);

		let right_block = Block::new().borders(Borders::LEFT);
		let right_area = right_block.inner(layout[1]);
		frame.render_widget(right_block, layout[1]);

		playlist_window(model, frame, layout[0]);
		command_window(model, frame, right_area);
	} else {
		playlist_window(model, frame, layout[0]);
	}

	let bottom_block = Block::new().borders(Borders::TOP);
	let bottom_area = bottom_block.inner(layout[1]);
	frame.render_widget(bottom_block, layout[1]);
	song_data(model, frame, bottom_area);
}

fn dont_draw_screen_replacement(frame: &mut Frame, area: Rect) {
	let area = area.centered(Constraint::Max(54), Constraint::Max(7));

	let center_block = Block::new().borders(Borders::ALL);
	let center_area = center_block.inner(area);
	frame.render_widget(center_block, area);

	let para = Paragraph::new(vec![
		Line::from("Rendering is currently disabled!").centered(),
		Line::from("Your inputs will still work though.").centered(),
		Line::from("You can enable rendering by pressing <Ctrl + R>.").centered(),
	]);
	let center_area = center_area.centered_vertically(Constraint::Max(3));
	frame.render_widget(para, center_area);
}

fn playlist_window(model: &Model, frame: &mut Frame, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Max(2), Constraint::Fill(1)])
		.split(area);

	let temp_mode_marker = match model.ctx.program_mode {
		ProgramMode::Main => "",
		ProgramMode::Temp => "TEMPORARY MODE      ",
	};

	let top_block = Block::new().borders(Borders::BOTTOM);
	let top_area = top_block.inner(layout[0]);
	frame.render_widget(top_block, layout[0]);
	let top_line = Line::from(format!(
		" {}list: {}      remaining: {}",
		temp_mode_marker,
		model.ctx.config.current_playlist,
		model.ctx.playlist.remaining.len()
	));
	frame.render_widget(top_line, top_area);

	let lines = model
		.ctx
		.playlist
		.remaining
		.iter()
		.take(layout[1].height as usize)
		.enumerate()
		.map(|(i, file)| {
			if i == layout[1].height as usize - 1 {
				return Line::from("   ...");
			}
			let song_name = file
				.file_name()
				.expect("Failed to get file name from the path.")
				.to_string_lossy();
			let mut line = Line::from(format!("{i:2} {song_name}"));
			if i == 0 {
				line = line.style(Style::new().black().on_white());
			}
			line
		})
		.collect::<Vec<_>>();
	frame.render_widget(Paragraph::new(lines), layout[1]);
}

fn command_window(model: &Model, frame: &mut Frame, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Max(2),
			Constraint::Max(1),
			Constraint::Fill(1),
		])
		.split(area);

	if let Some(cmd) = &model.current_command {
		frame.render_widget(
			Paragraph::new(format!(":{cmd}")).wrap(Wrap { trim: false }),
			layout[0],
		);
	}
	let cmd_out = Paragraph::new(&*model.ctx.cmd_out).wrap(Wrap { trim: false });
	frame.render_widget(cmd_out, layout[2]);
}

fn song_data(model: &Model, frame: &mut Frame, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(4),
			Constraint::Length(1),
			Constraint::Min(0),
		])
		.split(area);

	let style = Style::new().black().on_white();
	let top_bottom = Line::from(vec![
		Span::raw(" "),
		Span::styled("  ", style),
		Span::raw(" "),
	]);
	#[rustfmt::skip]
	let pause_str = if model.ctx.sink.is_paused() { " || " } else { " >> " };
	let text = Text::from(vec![
		top_bottom.clone(),
		Line::styled(pause_str, style),
		top_bottom,
	]);

	let progress_str = if let Some(song_duration) = model.ctx.get_current_duration() {
		display_duration_out_of(model.ctx.playlist.progress, song_duration)
	} else {
		display_duration(model.ctx.playlist.progress)
	};

	// FIXME displays the tags of the last song when there is no current song
	let tags_str = model
		.ctx
		.files
		.get(&model.current_song)
		.map_or(String::new(), |f| {
			let mut tags = f.tags.iter().cloned().collect::<Vec<_>>();
			tags.sort();
			tags.join(", ")
		});

	let volume = (100. * model.ctx.config.volume).round();
	let speed = (100. * model.ctx.config.speed).round() / 100.;

	frame.render_widget(text, layout[0]);
	frame.render_widget(
		Paragraph::new(vec![
			Line::raw(&model.current_song_name),
			Line::raw(format!(
				"volume: {volume}%   speed: x{speed}   tags: {tags_str}"
			)),
			Line::raw(progress_str),
		]),
		layout[2],
	);
}
