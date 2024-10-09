pub mod rackhost_client;

#[cfg(test)]
mod tests {
    use crate::rackhost_client;
    use crate::rackhost_client::RackhostClient;

    async fn get_client() -> RackhostClient<rackhost_client::Authed> {
        let username = env!("RACKHOST_USERNAME");
        let password = env!("RACKHOST_PASSWORD");
        //let client = RackhostClient::default().authenticate(username, password).await.unwrap();
        let client = RackhostClient::default()
            .authenticate(username, password)
            .await
            .expect("Failed to authenticate");
        client
    }

    #[tokio::test]
    async fn test_login() {
        let _client = get_client().await;
    }

    #[tokio::test]
    async fn test_domains() {
        let client = RackhostClient::default();
        //client.search_domain("testdomain").await.unwrap();
        //client.search_domain("othertestdomain").await.unwrap();
    }

    #[tokio::test]
    async fn test_dns_zones() {
        let client = get_client().await;
        let zones = client.get_dns_zones().await.unwrap();
        for zone in zones {
            println!("{:?}", zone);
        }
    }

    #[tokio::test]
    async fn test_dns_records() {
        let client = get_client().await;
        let zones = client.get_dns_zones().await.unwrap();
        let records = client.get_dns_records(&zones[0]).await.unwrap();
        for record in records {
            println!("{:?}", record);
        }
    }
}
