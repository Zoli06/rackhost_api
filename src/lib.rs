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
use reqwest::{Client, Error, Url};
use scraper::{CaseSensitivity, Element, Html};
use scraper::selector::CssLocalName;
use crate::endpoints::{BASE_URL, DOMAIN_SEARCH_PATH};

mod endpoints;

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
        let url = Url::parse_with_params(
            &format!("{}{}", BASE_URL, DOMAIN_SEARCH_PATH),
            &[("domainList", name.into())])
            .expect("Failed to parse URL");
        
        let response = self.client.get(url).send().await?;
        
        let body = response.text().await?;
        let doc = Html::parse_document(&body);

        let mut domains: Vec<DomainInfo> = vec![];

        // classnames
        let class_taken = CssLocalName::from("domain-taken");
        let class_free = CssLocalName::from("domain-free");

        // NOTES: data-domain attr always holds the domain.
        // if a domain is taken, it has a domain-taken class
        // if not, it has a domain-free class.
        // Domains owned by the user have a div instead of a form and just hold the data-domain attr

        let domain_hit_selector = scraper::Selector::parse("form[data-domain-search-res] div.domain-hit").unwrap(); // the second part skips the form.
        let domain_owned_selector = scraper::Selector::parse("div.domain-hit[data-domain]").unwrap();

        // the following selectors are ment to be used after the hit selector
        let domain_net_price_selector = scraper::Selector::parse("span[data-behavior=netPrice]").unwrap();
        let domain_gross_price_selector = scraper::Selector::parse("span[data-behavior=grossPrice]").unwrap();

        let domains_hit = doc.select(&domain_hit_selector);
        for element in domains_hit {
            let domain_name = element.attr("data-domain").unwrap();
            let mut domain_state = DomainState::Unknown;

            if element.has_class(&class_taken, CaseSensitivity::CaseSensitive) {
                domain_state = DomainState::Unavailable
            } else if element.has_class(&class_free, CaseSensitivity::CaseSensitive) {
                domain_state = DomainState::Available
            } else {
                // idk
                dbg!(&element);
            }

            let net_price = element.select(&domain_net_price_selector)
                .next()
                .expect("No net_price found")
                .text()
                .next()
                .expect("text element not found");

            let gross_price = element.select(&domain_gross_price_selector)
                .next()
                .expect("No net_price found")
                .text()
                .next()
                .expect("text element not found");

            let mut domain_info = DomainInfo::new(domain_name.to_owned(), domain_state);


            println!("{}: {:?}", domain_info.url, domain_info.domain_state);
            println!("{} ; {}", net_price, gross_price);
        }

        Ok(vec![])
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DomainState {
    Available,
    Unavailable,
    OwnedByUser,
    Unknown // idk how this might happen but still, its better than crashing
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub url: String,
    pub domain_state: DomainState,
    pub net_price: Option<f64>,
    pub gross_price: Option<f64>,
}

impl DomainInfo {
    pub fn new(url: String, domain_state: DomainState) -> Self {
        Self {
            url, domain_state,
            net_price: None, gross_price: None
        }
    }
    pub fn with_price(&mut self, net_price: f64, gross_price: f64) {
        self.net_price = Some(net_price);
        self.gross_price = Some(gross_price);
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
        client.search_domain("testdomain").await.unwrap();
        client.search_domain("othertestdomain").await.unwrap();
    }
}