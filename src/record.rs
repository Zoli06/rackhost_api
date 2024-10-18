use crate::auth::Authed;
use crate::base::{RackhostClient, BASE_URL};
use crate::zone::DnsZone;
use anyhow::{bail, Result};
use nutype::nutype;
use reqwest::Url;
use scraper::{Html, Selector};
use sealed::sealed;
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct HasId(pub u32);
#[derive(Debug, Clone)]
pub struct NoId;

#[sealed]
pub trait IdState {}
#[sealed]
impl IdState for HasId {}
#[sealed]
impl IdState for NoId {}

impl Display for HasId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn is_valid_ttl(value: &u32) -> bool {
    matches!(
        value,
        300 | 600
            | 900
            | 1800
            | 3600
            | 7200
            | 14400
            | 21600
            | 43200
            | 86400
            | 172800
            | 432000
            | 604800
    )
}

#[nutype(
    validate(predicate = is_valid_ttl),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)
)]
pub struct TTL(u32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumString, Display)]
pub enum RecordType {
    A,
    #[strum(serialize = "AAAA")]
    Aaaa,
    #[strum(serialize = "CNAME")]
    Cname,
    #[strum(serialize = "TXT")]
    Txt,
}

#[derive(Debug, Clone)]
pub struct DnsRecord<I: IdState> {
    pub host_name: String,
    pub record_type: RecordType,
    pub target: String,
    pub ttl: TTL,
    pub(super) id: I,
}

impl From<DnsRecord<HasId>> for DnsRecord<NoId> {
    fn from(record: DnsRecord<HasId>) -> Self {
        DnsRecord {
            host_name: record.host_name,
            record_type: record.record_type,
            target: record.target,
            ttl: record.ttl,
            id: NoId,
        }
    }
}

impl RackhostClient<Authed> {
    pub async fn get_dns_records(&self, zone: &DnsZone) -> Result<Vec<DnsRecord<HasId>>> {
        let url = BASE_URL.join(format!("dnsZone/{}", zone.id).as_str())?;
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
                id: HasId(id.parse().expect("Failed to parse id")),
                host_name,
                record_type: RecordType::from_str(&record_type).expect("Invalid record type"),
                target: value,
                ttl: TTL::try_new(ttl).expect("Invalid TTL"),
            });
        }

        Ok(dns_records)
    }

    async fn create_or_update_dns_record(&self, url: Url, record: &DnsRecord<NoId>) -> Result<()> {
        let response = self
            .client
            .post(url)
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&[
                ("rackhost-csrf", self.get_csrf_token().await?),
                // TODO: do something with the subdomain
                (
                    "DnsRecordForm[name]",
                    String::from(
                        record
                            .host_name
                            .split('.')
                            .next()
                            .expect("No subdomain found"),
                    ),
                ),
                ("DnsRecordForm[type]", record.record_type.to_string()),
                ("DnsRecordForm[target]", record.target.clone()),
                ("DnsRecordForm[ttl]", record.ttl.to_string()),
            ])
            .send()
            .await?;

        // Parse response body json
        let body = response.text().await?;
        let json = serde_json::from_str::<serde_json::Value>(&body)?;

        // Parse html from message key
        let message = json["message"].as_str().expect("No message found");
        let message_html = Html::parse_fragment(message);
        let selector = Selector::parse("div.alert-success").expect("Invalid selector");
        let success = message_html.select(&selector).next().is_some();
        if !success {
            bail!("Failed to update record");
        }

        Ok(())
    }

    pub async fn create_dns_record(&self, zone: &DnsZone, record: &DnsRecord<NoId>) -> Result<()> {
        let mut url = BASE_URL.join("dnsRecord/createOther")?;
        url.set_query(Some(format!("dnsZoneId={}", zone.id).as_str()));
        self.create_or_update_dns_record(url, record)
            .await
            .expect("Failed to create record");
        Ok(())
    }

    pub async fn update_dns_record(&self, record: &DnsRecord<HasId>) -> Result<()> {
        let url = BASE_URL.join(format!("dnsRecord/updateOther/{}", record.id).as_str())?;
        let record = DnsRecord::<NoId>::from(record.clone());
        self.create_or_update_dns_record(url, &record)
            .await
            .expect("Failed to update record");
        Ok(())
    }
}
