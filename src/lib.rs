// Write an API wrapper for a website named rackhost.hu
// Every request contains csrf token
// The API should be able to:
// - List all dns zones
// - List all records in a dns zone
// - Create a new record in a dns zone
// - Update a record in a dns zone
// - Delete a record in a dns zone

// ^^  bro really tried to use Copilot ^^

use std::marker::PhantomData;
use reqwest::{Client, Error};
use scraper::Html;
use domains::DomainInfo;
mod endpoints;
mod domains;

struct Authed;
struct NotAuthed;

#[derive(Debug)]
pub struct RackhostClient<L /* Login State (or validation) */> {
    _phantom_state: PhantomData<L>,
    client: Client,
}

impl RackhostClient<NotAuthed> {
    pub fn new(client: Client) -> Self {
        Self {
            _phantom_state: PhantomData::default(),
            client
        }
    }

    pub async fn authenticate(mut self, username: impl Into<String>, password: impl Into<String>) -> anyhow::Result<RackhostClient<Authed>> {
        let csrf_token = Self::get_csrf_token(&mut self).await?;

        let response = self.client.post(endpoints::BASE_URL.to_owned() + endpoints::LOGIN_PATH)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", username.into()),
                ("LoginForm[password]", password.into())
            ])
            .send().await?;

        if response.url().as_str() == endpoints::BASE_URL.to_owned() + endpoints::LOGIN_PATH {
            anyhow::bail!("Login failed");
        }

        Ok(RackhostClient::<Authed>{
            _phantom_state: PhantomData::default(),
            client: self.client,
        })
    }
}

impl RackhostClient<Authed> {
}

impl<L> RackhostClient<L> { // shared behaviour
    pub async fn search_domain(&self, name: impl Into<String>) -> anyhow::Result<Vec<DomainInfo>> {
        domains::search_domain(self, name).await
    }

    pub async fn get_csrf_token(&mut self) -> Result<String, Error> {
        let response = self.client.get(endpoints::BASE_URL.to_owned() + "/site/login").send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let path = scraper::Selector::parse("input[name=rackhost-csrf]").unwrap();
        let csrf_token = document.select(&path).next().unwrap().value().attr("value").unwrap();

        Ok(csrf_token.to_owned())
    }
}

impl Default for RackhostClient<NotAuthed> {
    fn default() -> Self {
        Self {
            _phantom_state: Default::default(),
            client: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let username = option_env!("TEST_USERNAME").expect("No username given for test");
        let password = option_env!("TEST_PASSWORD").expect("No password given for test");
        let rackhost_client = RackhostClient::default().authenticate(username, password).await;
        let _cli = match rackhost_client {
            Ok(client) => client,
            Err(err) => {
                dbg!(&err);
                assert!(false);
                return;
            }
        };
        
        assert!(true)
        
    }
    
    #[tokio::test]
    async fn test_domains() {
        let client = RackhostClient::default();
        let domains = client.search_domain("test_domain").await;
        println!("{:#?}", domains)
    }
}