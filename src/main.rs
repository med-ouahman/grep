

use grep::ExitCode;
use std::env;
use std::process;

fn main() {
	let args: Vec<String> = env::args().collect();

	let status = match grep::run(&args) {
		ExitCode::Success => 0,
		ExitCode::Error => 2,
	};

	process::exit(status);
}
