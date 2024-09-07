pub struct Config {
    host: String,
    port: String
}

impl Config {
    pub fn new(host: String, port: String) -> Self {
        Self { host, port }
    }

    pub fn load(&self) -> (String, String) {
        (self.host.clone(), self.port.clone())
    }
}