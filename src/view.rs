use crate::{
	Model,
	duration::{display_duration, display_duration_out_of},
};
use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Style, Stylize},
	text::{Line, Span, Text},
	widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn view(model: &Model, frame: &mut Frame) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Fill(1), Constraint::Length(4)])
		.split(frame.area());

	command_window(model, frame, layout[0]);

	let bottom_block = Block::new().borders(Borders::TOP);
	let bottom_area = bottom_block.inner(layout[1]);
	frame.render_widget(bottom_block, layout[1]);
	song_data(model, frame, bottom_area);
}

fn command_window(model: &Model, frame: &mut Frame, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Min(1),
			Constraint::Length(1),
			Constraint::Fill(1),
		])
		.split(area);

	let text = &model.currently_typing;
	frame.render_widget(
		Paragraph::new(format!("Command: {text}")).wrap(Wrap { trim: false }),
		layout[0],
	);
	frame.render_widget(Text::raw(&model.command_output), layout[2]);
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
	let remaining = model.ctx.playlist.remaining.len();

	frame.render_widget(text, layout[0]);
	frame.render_widget(
		Paragraph::new(vec![
			Line::raw(format!("Song: {}", model.current_song_name)),
			Line::raw(format!("Remaining Songs: {remaining}")),
			Line::raw(progress_str),
		]),
		layout[2],
	);
}
