// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

fn main() -> Result<(), Box<dyn std::error::Error>> {
	if std::env::var("CARGO_CFG_TARGET_OS")? == "windows" {
		winresource::WindowsResource::new()
			.set_icon("assets/icon.ico")
			.compile()?;
	}
	Ok(())
}
