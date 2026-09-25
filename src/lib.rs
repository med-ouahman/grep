
mod exit;
mod cli;

pub use exit::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    
    let conf = match cli::args::parse_args(args) {
        Ok(conf) => conf,
        Err(_) => return ExitCode::Error
    };

    conf.display();

    ExitCode::Success
}

