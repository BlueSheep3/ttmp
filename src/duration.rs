use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DurationParseError {
	#[error("Failed while parsing Float: {0}")]
	ParseFloat(#[from] std::num::ParseFloatError),

	#[error("impossible duration: {0}")]
	ImpossibleDuration(f32),
	#[error("empty duration part")]
	EmptyDurationPart,
	#[error("unknown unit: {0}")]
	UnknownUnit(char),
}

pub fn parse_duration(duration_str: &str) -> Result<Duration, DurationParseError> {
	use DurationParseError::{EmptyDurationPart, ImpossibleDuration, UnknownUnit};

	fn f32_to_duration(f: f32) -> Result<Duration, DurationParseError> {
		Duration::try_from_secs_f32(f).map_err(|_| ImpossibleDuration(f))
	}

	let mut duration = Duration::ZERO;
	for part in duration_str.split(' ') {
		let unit = part.chars().last().ok_or(EmptyDurationPart)?;
		if unit.is_ascii_digit() {
			// default to seconds
			let num = part.parse::<f32>()?;
			duration += f32_to_duration(num)?;
			continue;
		}
		let num = part[0..part.len() - 1].parse::<f32>()?;
		match unit {
			's' => duration += f32_to_duration(num)?,
			'm' => duration += f32_to_duration(num * 60.)?,
			'h' => duration += f32_to_duration(num * 60. * 60.)?,
			_ => return Err(UnknownUnit(unit)),
		}
	}
	Ok(duration)
}

pub fn display_duration(duration: Duration) -> String {
	let s = duration.as_secs_f32();
	let secs = s % 60.;
	let mins = (s / 60.) % 60.;
	let hours = (s / 3600.) % 60.;
	format!("{:02.0}:{:02.0}:{:05.2}", hours, mins, secs)
}
