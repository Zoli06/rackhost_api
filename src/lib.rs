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
use anyhow::bail;
use reqwest::{Client, Url};
use scraper::{Html, Selector};

struct Authed;
struct NotAuthed;

const BASE_URL: &str = "https://www.rackhost.hu";

#[derive(Debug)]
pub struct RackhostClient<L /* Login State (or validation) */> {
    _phantom_state: PhantomData<L>,
    client: Client,
}

impl RackhostClient<NotAuthed> {
    pub fn new(client: Client) -> Self {
        Self {
            _phantom_state: PhantomData,
            client
        }
    }

    pub async fn authenticate(self, username: impl Into<String>, password: impl Into<String>) -> anyhow::Result<RackhostClient<Authed>> {
        let url = format!("{}/site/login", BASE_URL);

        let csrf_token = self.get_csrf_token().await?;
        let response = self
            .client
            .post(url)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", username.into()),
                ("LoginForm[password]", password.into())
            ])
            .send()
            .await?;

        if !response.status().is_redirection() {
            bail!("Login failed");
        }

        Ok(RackhostClient {
            _phantom_state: PhantomData,
            client: self.client
        })
    }
}

impl RackhostClient<Authed> {
}

impl<L> RackhostClient<L> { // shared behaviour
    pub async fn search_domain(&self, name: impl Into<String>) -> anyhow::Result<Vec<DomainInfo>> {
        unimplemented!();
        let url = Url::parse_with_params(
            format!("{}/domain", BASE_URL).as_str(),
            &[("domainList", name.into())])
            .expect("Failed to parse URL");
        
        let response = self.client.get(url).send().await?;
        
        let body = response.text().await?;
        let doc = Html::parse_document(&body);
        
        let mut domains: Vec<DomainInfo> = vec![];
        
        let domain_hit_selector = scraper::Selector::parse("form[data-domain-search-res]").unwrap();
        let domain_owned_selector = scraper::Selector::parse("div.domain-hit[data-domain]").unwrap();
        let domains_hit = doc.select(&domain_hit_selector);
        //domains_hit.next().unwrap().has
        
        
        
        let domain_search_name_selector = scraper::Selector::parse("span.search-words").unwrap();
        let mut search = doc.select(&domain_search_name_selector);
        let name = search.next()
            .unwrap()
            .text()
            .next()
            .unwrap();
        
        // found (same as name, TODO: need to validate that the matches are the same. )
        
        println!("{}", name);
        
        let domain_ext_selector = scraper::Selector::parse("div.domain-hit span.tld").unwrap();
        let extensions = doc.select(&domain_ext_selector);
        for ext in extensions {
            println!("{}", ext.text().next().unwrap());
        }
        
        // get availability
        
        let domain_availability_selector = scraper::Selector::parse("div.domain-hit div.avail").unwrap();
        let avails = doc.select(&domain_availability_selector);
        for avail_txt in avails {
            println!("{}", avail_txt.text().next().unwrap())
        }
        
        Ok(vec![])
    }

    // TODO: check if csrf from login is also valid for other endpoints
    async fn get_csrf_token(&self) -> anyhow::Result<String> {
        let url = format!("{}/site/login", BASE_URL);
        let response = self
            .client
            .get(url)
            .send()
            .await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let path = Selector::parse("input[name=rackhost-csrf]").expect("Invalid selector");
        let csrf_token = document
            .select(&path)
            .next()
            .expect("No csrf input element found")
            .value()
            .attr("value")
            .expect("No csrf token found");
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DomainState {
    Available,
    Unavailable,
    OwnedByUser
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub url: Url,
    pub domain_state: DomainState
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let username = option_env!("TEST_USERNAME").expect("No username given for test");
        let password = option_env!("TEST_PASSWORD").expect("No password given for test");
        let rackhost_client = RackhostClient::default().authenticate(username, password).await;
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
    
    #[tokio::test]
    async fn test_domains() {
        let client = RackhostClient::default();
        //client.search_domain("testdomain").await.unwrap();
        //client.search_domain("othertestdomain").await.unwrap();
    }
}