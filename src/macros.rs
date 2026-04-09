// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

#[macro_export]
macro_rules! defer {
	($($r:tt)*) => {
		let _defer = {
			struct Defer;
			impl ::std::ops::Drop for Defer {
				fn drop(&mut self) {
					$($r)*
				}
			}
			Defer
		};
	};
}
