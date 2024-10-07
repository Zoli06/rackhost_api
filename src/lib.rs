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
use reqwest::Client;
mod endpoints;

struct Authed;
struct NotAuthed;
pub struct RackHostClient<LS /* Login State (or validation) */> {
    _phantom_state: PhantomData<LS>,

    username: String,
    password: String,
    user_agent: String,

    client: Client,
}

mod utils {
    use reqwest::{Client, Error};
    use scraper::Html;
    use crate::endpoints;

    pub async fn get_csrf_token(client: &Client) -> Result<String, Error> {
        let response = client.get(endpoints::BASE_URL.to_owned() + "/site/login").send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let path = scraper::Selector::parse("input[name=rackhost-csrf]").unwrap();
        let csrf_token = document.select(&path).next().unwrap().value().attr("value").unwrap();
        
        Ok(csrf_token.to_owned())
    }
    
    pub async fn authenticate(client: &mut Client, username: String, password: String) -> anyhow::Result<()> {
        let csrf_token = get_csrf_token(client).await?;
        let response = client.post(endpoints::BASE_URL.to_owned() + endpoints::LOGIN_PATH)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", username),
                ("LoginForm[password]", password)
            ])
            .send().await?;

        if response.url().as_str() == endpoints::BASE_URL.to_owned() + endpoints::LOGIN_PATH {
            anyhow::bail!("Login failed");
        }

        Ok(())
    }
}

impl RackHostClient<NotAuthed> {
    pub fn new(username: String, password: String) -> Self {
        Self {
            _phantom_state: PhantomData::default(),
            username,
            password,
            user_agent: "Firefox".to_string(), // current default
            client: Client::new(),
        }
    }
    pub async fn authenticate(mut self) -> anyhow::Result<RackHostClient<Authed>> {
        utils::authenticate(&mut self.client, self.username.clone(), self.password.clone()).await?;
        Ok(RackHostClient::<Authed>{
            _phantom_state: PhantomData::default(),
            username: self.username,
            password: self.password,
            user_agent: self.user_agent,
            client: self.client,
        })
    }
}

impl RackHostClient<Authed> {
    pub async fn login(username: String, password: String) -> anyhow::Result<Self> {
        let mut client = Self {
            _phantom_state: PhantomData::default(),
            username,
            password,
            user_agent: "Firefox".to_owned(),
            client: Client::new()
        };
        
        utils::authenticate(&mut client.client, client.username.clone(), client.password.clone()).await?;
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let username = option_env!("TEST_USERNAME").expect("No username given for test");
        let password = option_env!("TEST_PASSWORD").expect("No password given for test");
        let rackhost_client = RackHostClient::login(username.to_owned(), password.to_owned()).await;
        let cli = match rackhost_client {
            Ok(client) => client,
            Err(err) => {
                dbg!(&err);
                assert!(false);
                return;
            }
        };
        
        assert!(true)
        
    }
}
