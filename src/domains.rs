use reqwest::Url;
use rusty_money::{iso, Money};
use rusty_money::iso::Currency;
use scraper::{CaseSensitivity, Element, Html};
use scraper::selector::CssLocalName;
use crate::endpoints::{BASE_URL, DOMAIN_SEARCH_PATH};
use crate::RackhostClient;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DomainState {
    Available,
    Unavailable,
    RegisteredOnRackhost,
    Unknown // IDK how this might happen but still, it's better than crashing
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub url: String,
    pub domain_state: DomainState,
    pub domain_pricing: Option<DomainPricing>,
}

#[derive(Debug, Clone)]
pub struct DomainPricing { /// Todo: refactor naming of these prices
    pub registration_price: Option<Money<'static, Currency>>,
    pub price_one_year: Money<'static, Currency>,
    pub price_two_years: Money<'static, Currency>,
    pub price_three_years: Money<'static, Currency>,
    pub price_four_years: Money<'static, Currency>,
    pub price_five_years: Money<'static, Currency>,
    pub price_six_years: Money<'static, Currency>,
    pub price_seven_years: Money<'static, Currency>,
    pub price_eight_years: Money<'static, Currency>,
    pub price_nine_years: Money<'static, Currency>,
    pub price_ten_years: Option<Money<'static, Currency>>,
}

#[derive(Debug, Clone, Default)]
pub struct DomainPricingBuilder { /// Todo: refactor naming of these prices
    pub registration_price: Option<Money<'static, Currency>>,
    pub price_one_year: Option<Money<'static, Currency>>,
    pub price_two_years: Option<Money<'static, Currency>>,
    pub price_three_years: Option<Money<'static, Currency>>,
    pub price_four_years: Option<Money<'static, Currency>>,
    pub price_five_years: Option<Money<'static, Currency>>,
    pub price_six_years: Option<Money<'static, Currency>>,
    pub price_seven_years: Option<Money<'static, Currency>>,
    pub price_eight_years: Option<Money<'static, Currency>>,
    pub price_nine_years: Option<Money<'static, Currency>>,
    pub price_ten_years: Option<Money<'static, Currency>>,
}

impl DomainPricingBuilder {
    pub fn new() -> Self {
        DomainPricingBuilder::default()
    }

    pub fn set_by_quantity(&mut self, price: String, quantity: u64) {
        let money: Option<Money<'static, Currency>> = Some(Money::from_str(&price, iso::HUF).expect("Money creation error")); // TODO: Better error
        match quantity {
            0 => self.registration_price    = money,
            1 => self.price_one_year        = money,
            2 => self.price_two_years       = money,
            3 => self.price_three_years     = money,
            4 => self.price_four_years      = money,
            5 => self.price_five_years      = money,
            6 => self.price_six_years       = money,
            7 => self.price_seven_years     = money,
            8 => self.price_eight_years     = money,
            9 => self.price_nine_years      = money,
            10 => self.price_ten_years      = money,
            _ => { unreachable!() }
        }
    }

    pub fn build(self) -> anyhow::Result<DomainPricing> {
        Ok(
            DomainPricing {
                registration_price: self.registration_price,
                price_one_year:     self.price_one_year.ok_or(anyhow::anyhow!("Field missing"))?,
                price_two_years:    self.price_two_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_three_years:  self.price_three_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_four_years:   self.price_four_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_five_years:   self.price_five_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_six_years:    self.price_six_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_seven_years:  self.price_seven_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_eight_years:  self.price_eight_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_nine_years:   self.price_nine_years.ok_or(anyhow::anyhow!("Field missing"))?,
                price_ten_years:    self.price_ten_years,
        })
    }
}

impl DomainInfo {
    pub fn new(url: String, domain_state: DomainState) -> Self {
        Self {
            url, domain_state,
            domain_pricing: None
        }
    }
    pub fn with_price(&mut self, domain_pricing: DomainPricing) {
        self.domain_pricing = Some(domain_pricing)
    }
}

pub async fn search_domain<A>(rackhost_client: &RackhostClient<A>, name: impl Into<String>) -> anyhow::Result<Vec<DomainInfo>> {
    let url = Url::parse_with_params(
        &format!("{}{}", BASE_URL, DOMAIN_SEARCH_PATH),
        &[("domainList", name.into())])
        .expect("Failed to parse URL");

    let response = rackhost_client.client.get(url).send().await?;

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

    let domain_hit_selector = scraper::Selector::parse("div.domain-hit[data-domain]").expect("Selector parse failed"); // the second part skips the form.
    //let domain_registered_selector = scraper::Selector::parse("div.domain-hit[data-domain]:not(div.domain-free):not(div.domain-taken)").expect("Selector parse failed");

    // the following selectors are meant to be used after the hit selector
    let domain_price_option_selector = scraper::Selector::parse("option[data-json]").expect("Selector parse failed");

    let domains_hit = doc.select(&domain_hit_selector);
    for element in domains_hit {
        let domain_name = element.attr("data-domain").unwrap();
        let domain_state;

        if element.has_class(&class_taken, CaseSensitivity::CaseSensitive) {
            domain_state = DomainState::Unavailable
        } else if element.has_class(&class_free, CaseSensitivity::CaseSensitive) {
            domain_state = DomainState::Available
        } else {
            // owned by rackhost user.
            domain_state = DomainState::RegisteredOnRackhost
        }

        if domain_state == DomainState::RegisteredOnRackhost {
            domains.push(DomainInfo::new(domain_name.to_owned(), domain_state));
            continue;
        }

        // get pricing info
        let price_options = element.select(&domain_price_option_selector);
        let mut prices = vec![];

        for pricing_option in price_options {
            // get the data-json attr value
            let json_data = pricing_option.attr("data-json").expect("json info of pricing option somehow missing.");
            let data: serde_json::Value = serde_json::from_str(json_data).expect("Failed to parse JSON data");

            // This panics if the wrong data is received!!!
            let value = data["gtag"]["value"].as_i64().expect("Value field failed to parse").to_string();
            let quantity = data["gtag"]["items"][0]["quantity"].as_u64().unwrap();
            let currency = data["gtag"]["currency"].as_str().unwrap().to_owned();

            prices.push((value, currency, quantity));
        }

        let mut domain_info = DomainInfo::new(domain_name.to_owned(), domain_state);

        let mut pricing = DomainPricingBuilder::new();
        for (price, currency, quantity) in prices {
            assert_eq!(&currency, "HUF"); // others not implemented yet
            pricing.set_by_quantity(price, quantity);
        }

        domain_info.with_price(pricing.build()?);
        domains.push(domain_info);
    }

    Ok(domains)
}