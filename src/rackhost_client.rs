use anyhow::bail;
use anyhow::Result;
use refined_type::result::Error;
use refined_type::rule::Rule;
use refined_type::Refined;
use reqwest::{Client, ClientBuilder, Url};
use scraper::{Html, Selector};
use std::marker::PhantomData;
use std::str::FromStr;
use std::string::ParseError;
use strum_macros::EnumString;

const BASE_URL: &str = "https://www.rackhost.hu";

//region Types
pub type TTL = Refined<IsTTL>;
//endregion

//region Structs
pub struct Authed;
pub struct NotAuthed;

#[derive(Debug)]
pub struct RackhostClient<L /* Login State (or validation) */> {
    _phantom_state: PhantomData<L>,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub url: Url,
    pub domain_state: DomainState,
}

#[derive(Debug, Clone)]
pub struct DnsZone {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct DnsRecord {
    pub host_name: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: TTL,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct IsTTL;
// endregion

//region Enums
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DomainState {
    Available,
    Unavailable,
    OwnedByUser,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumString)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    TXT,
}
//endregion

//region Implementations
impl Rule for IsTTL {
    type Item = u32;
    fn validate(item: Self::Item) -> Result<Self::Item, Error<Self::Item>> {
        match item {
            300 | 600 | 900 | 1800 | 3600 | 7200 | 14400 | 21600 | 43200 | 86400 | 172800
            | 432000 | 604800 => Ok(item),
            _ => Err(Error::new(item, "Invalid TTL value")),
        }
    }
}

impl RackhostClient<NotAuthed> {
    pub fn new(client_builder: ClientBuilder) -> Self {
        Self {
            _phantom_state: PhantomData,
            client: client_builder
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Failed to create client"),
        }
    }

    pub async fn authenticate(
        self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<RackhostClient<Authed>> {
        let url = format!("{}/site/login", BASE_URL);

        let csrf_token = self.get_csrf_token().await?;
        let response = self
            .client
            .post(url)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", username.into()),
                ("LoginForm[password]", password.into()),
            ])
            .send()
            .await?;

        if !response.status().is_redirection() {
            bail!("Login failed");
        }

        Ok(RackhostClient {
            _phantom_state: PhantomData,
            client: self.client,
        })
    }
}

impl Default for RackhostClient<NotAuthed> {
    fn default() -> Self {
        Self::new(Client::builder())
    }
}

impl RackhostClient<Authed> {
    pub async fn get_dns_zones(&self) -> Result<Vec<DnsZone>> {
        let url =
            Url::parse(format!("{}/dnsZone", BASE_URL).as_str()).expect("Failed to parse URL");
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        let doc = Html::parse_document(&body);

        let zone_selector = Selector::parse("#dns-zone-grid table tbody tr td:nth-child(1) a")
            .expect("Invalid selector");
        let zones = doc.select(&zone_selector);
        let mut dns_zones = vec![];
        for zone in zones {
            let url = zone.value().attr("href").expect("No href found");
            let id = url.split('/').last().expect("No id found").to_owned();
            let name = zone.text().next().expect("No name found").to_owned();
            dns_zones.push(DnsZone { id, name });
        }

        Ok(dns_zones)
    }

    pub async fn get_dns_records(&self, zone: &DnsZone) -> Result<Vec<DnsRecord>> {
        let url = Url::parse(format!("{}/dnsZone/{}", BASE_URL, zone.id).as_str())
            .expect("Failed to parse URL");
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        let doc = Html::parse_document(&body);

        let record_selector =
            Selector::parse("#dns-record-grid-0 table tbody tr").expect("Invalid selector");
        let records = doc.select(&record_selector);
        let mut dns_records = vec![];
        for record in records {
            let host_name = record
                .select(&Selector::parse("td:nth-child(1)").expect("Invalid selector"))
                .next()
                .expect("No name found")
                .text()
                .next()
                .expect("No name found")
                .to_owned();
            let record_type = record
                .select(&Selector::parse("td:nth-child(2)").expect("Invalid selector"))
                .next()
                .expect("No record type found")
                .text()
                .next()
                .expect("No record type found")
                .to_owned();
            let value = record
                .select(&Selector::parse("td:nth-child(3)").expect("Invalid selector"))
                .next()
                .expect("No value found")
                .text()
                .next()
                .expect("No value found")
                .to_owned();
            let ttl = record
                .select(&Selector::parse("td:nth-child(4)").expect("Invalid selector"))
                .next()
                .expect("No ttl found")
                .text()
                .next()
                .expect("No ttl found")
                .parse::<u32>()
                .expect("Failed to parse ttl");
            let id = record
                .select(&Selector::parse("td:nth-child(5) a").expect("Invalid selector"))
                .next()
                .expect("No id found")
                .value()
                .attr("href")
                .expect("No href found")
                .split('/')
                .last()
                .expect("No id found")
                .to_owned();
            dns_records.push(DnsRecord {
                id,
                host_name,
                record_type: RecordType::from_str(&record_type).expect("Invalid record type"),
                value,
                ttl: TTL::new(ttl).expect("Invalid TTL"),
            });
        }

        Ok(dns_records)
    }
}

impl<L> RackhostClient<L> {
    pub async fn search_domain(&self, name: impl Into<String>) -> Result<Vec<DomainInfo>> {
        unimplemented!();
        let url = Url::parse_with_params(
            format!("{}/domain", BASE_URL).as_str(),
            &[("domainList", name.into())],
        )
        .expect("Failed to parse URL");

        let response = self.client.get(url).send().await?;

        let body = response.text().await?;
        let doc = Html::parse_document(&body);

        let mut domains: Vec<DomainInfo> = vec![];

        let domain_hit_selector = scraper::Selector::parse("form[data-domain-search-res]").unwrap();
        let domain_owned_selector =
            scraper::Selector::parse("div.domain-hit[data-domain]").unwrap();
        let domains_hit = doc.select(&domain_hit_selector);
        //domains_hit.next().unwrap().has

        let domain_search_name_selector = scraper::Selector::parse("span.search-words").unwrap();
        let mut search = doc.select(&domain_search_name_selector);
        let name = search.next().unwrap().text().next().unwrap();

        // found (same as name, TODO: need to validate that the matches are the same. )

        println!("{}", name);

        let domain_ext_selector = scraper::Selector::parse("div.domain-hit span.tld").unwrap();
        let extensions = doc.select(&domain_ext_selector);
        for ext in extensions {
            println!("{}", ext.text().next().unwrap());
        }

        // get availability

        let domain_availability_selector =
            scraper::Selector::parse("div.domain-hit div.avail").unwrap();
        let avails = doc.select(&domain_availability_selector);
        for avail_txt in avails {
            println!("{}", avail_txt.text().next().unwrap())
        }

        Ok(vec![])
    }

    // TODO: check if csrf from login is also valid for other endpoints
    async fn get_csrf_token(&self) -> Result<String> {
        let url = format!("{}/site/login", BASE_URL);
        let response = self.client.get(url).send().await?;
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
//endregion
