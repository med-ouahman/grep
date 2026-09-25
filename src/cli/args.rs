
use super::config::Config;
use super::config::ConfigError;

pub(crate) fn parse_args(args: &[String]) -> Result<Config, ConfigError> {

    if args.len() < 2 {
        Config::help();
        return Err(ConfigError::Agrument);
    }

    let something = args[1].clone();

    Ok(Config::new(something))
}
