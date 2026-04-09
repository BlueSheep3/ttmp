// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

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
	let mins = ((s / 60.) % 60.).floor();
	let hours = ((s / 3600.) % 60.).floor();
	format!("{hours:02.0}:{mins:02.0}:{secs:05.2}")
}

pub fn display_duration_out_of(duration: Duration, out_of: Duration) -> String {
	let secs1 = duration.as_secs_f32();
	let secs2 = out_of.as_secs_f32();
	let percent = ((secs1 / secs2) * 100.).clamp(0., 100.);
	let s1 = display_duration(duration);
	let s2 = display_duration(out_of);
	format!("{percent:03.0}%  =  {s1}  /  {s2}")
}
