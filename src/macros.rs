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
