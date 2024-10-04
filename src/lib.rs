// Write an API wrapper for a website named rackhost.hu
// Every request contains csrf token
// The API should be able to:
// - List all dns zones
// - List all records in a dns zone
// - Create a new record in a dns zone
// - Update a record in a dns zone
// - Delete a record in a dns zone

use crate::config::Config;
use reqwest::{Client, Error};
use scraper::Html;


mod config;
mod endpoints;

const BASE_URL: &str = "https://rackhost.hu";

pub struct Rackhost {
    config: Config,
    client: Client,
}

impl Rackhost {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    // TODO: check if csrf from login is also valid for other requests
    async fn get_csrf_token(&self) -> Result<String, Error> {
        let response = self.client.get(BASE_URL.to_owned() + "/site/login").send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let path = scraper::Selector::parse("input[name=rackhost-csrf]").unwrap();
        let csrf_token = document.select(&path).next().unwrap().value().attr("value").unwrap();
        Ok(csrf_token.to_owned())
    }

    pub async fn login(&self) -> anyhow::Result<()> {
        const LOGIN_URL: &str = "/site/login";

        let csrf_token = self.get_csrf_token().await?;
        let response = self.client.post(BASE_URL.to_owned() + LOGIN_URL)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", self.config.username.clone()),
                ("LoginForm[password]", self.config.password.clone())
            ])
            .send().await?;

        if response.url().as_str() == BASE_URL.to_owned() + LOGIN_URL {
            anyhow::bail!("Login failed");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let config = Config::new("Firefox".to_owned(), "username".to_owned(), "pass".to_owned());
        let rackhost = Rackhost::new(config);
        let result = rackhost.login().await;
        dbg!(&result);
        assert!(result.is_ok());
    }
}
