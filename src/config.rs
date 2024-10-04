pub struct Config {
    pub user_agent: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn new(user_agent: String, username: String, password: String) -> Self {
        Self {
            user_agent,
            username,
            password
        }
    }
}