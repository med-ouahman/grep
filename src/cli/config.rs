

pub(crate) enum ConfigError {
    Agrument
}



pub(crate) struct Config {
    pattern: String
}

impl Config {
    
    pub(crate) fn new(p: String) -> Self {
        Self { pattern: p }
    }
    
    pub(crate) fn display(&self) {
        println!("{}", self.pattern);
    }

    pub(crate) fn help() {
        println!("Usage:");
        println!("grep PATTERN [OPTIONS]");
    }
}
