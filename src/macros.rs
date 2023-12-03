#[macro_export]
macro_rules! readln {
	($($($r:tt)+)?) => {{
		$(
			print!($($r)+);
			use std::io::{stdout, Write};
			stdout().flush().unwrap();
		)?

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();
		if input.ends_with('\n') { input.pop(); }
		if input.ends_with('\r') { input.pop(); }
		input
	}};
}
