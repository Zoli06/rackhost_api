use crate::auth::Authed;
use crate::base::{RackhostClient, BASE_URL};
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DnsZone {
    pub id: String,
    pub name: String,
}

impl RackhostClient<Authed> {
    pub async fn get_dns_zones(&self) -> anyhow::Result<Vec<DnsZone>> {
        let url = BASE_URL.join("/dnsZone")?;
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
}
