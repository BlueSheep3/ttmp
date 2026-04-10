// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

fn main() -> std::io::Result<()> {
	#[cfg(target_os = "windows")]
	{
		winres::WindowsResource::new()
			.set_icon("assets/icon.ico")
			.compile()?;
	}
	Ok(())
}
