pub mod rackhost_client;

#[cfg(test)]
mod tests {
    use crate::rackhost_client::RackhostClient;

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