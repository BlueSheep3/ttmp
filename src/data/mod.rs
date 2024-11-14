pub mod config;
pub mod context;
pub mod files;
pub mod playlist;

use std::path::PathBuf;

fn get_savedata_path() -> PathBuf {
	#[cfg(not(target_os = "windows"))]
	compile_error!("get_savedata_path() only works on windows");

	let path = std::env::var("APPDATA").expect("APPDATA not found");
	let path = PathBuf::from(path);
	let path = path.parent().expect("appdata doesn't have a parent");
	path.join("LocalLow/BlueSheep3/Music Player")
}
