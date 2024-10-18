use crate::auth::AuthState;
use crate::base::{RackhostClient, BASE_URL};
use scraper::{Html, Selector};

impl<A: AuthState> RackhostClient<A> {
    pub(crate) async fn get_csrf_token(&self) -> anyhow::Result<String> {
        let url = BASE_URL.join("/site/login")?;
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
