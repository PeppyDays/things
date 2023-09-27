pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn new() -> Self {
        Self { port: 8080 }
    }
}

pub async fn get_config() -> Config {
    Config::new()
}
