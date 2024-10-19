pub mod auth;
pub mod base;
pub mod builder;
pub mod csrf;
pub mod record;
pub mod zone;

#[cfg(test)]
mod tests {
    use crate::auth::Authed;
    use crate::base::RackhostClient;
    use crate::record::{DnsRecord, NoId, RecordType, TTL};
    use dotenv::dotenv;
    use reqwest::{Client, Proxy};
    use std::env::var;
    use std::time::Duration;
    use tokio::sync::OnceCell;
    use tokio::test;

    static CLIENT: OnceCell<RackhostClient<Authed>> = OnceCell::const_new();

    async fn get_client() -> &'static RackhostClient<Authed> {
        // load env
        dotenv().ok();
        CLIENT
            .get_or_init(|| async {
                let username = var("RACKHOST_USERNAME").expect("RACKHOST_USERNAME not set");
                let password = var("RACKHOST_PASSWORD").expect("RACKHOST_PASSWORD not set");

                let reqwest_client_builder = Client::builder()
                    .proxy(Proxy::all("http://localhost:8080").expect("Failed to create proxy"));

                let rackhost_client = RackhostClient::builder()
                    .client_builder(reqwest_client_builder)
                    .rate_limit_from_duration(Duration::from_secs(3))
                    .build();

                rackhost_client
                    .authenticate(username, password)
                    .await
                    .expect("Failed to authenticate")
            })
            .await
    }

    #[test]
    async fn test_dns_zones() {
        get_client()
            .await
            .get_dns_zones()
            .await
            .expect("Failed to get dns zones");
    }

    #[test]
    async fn test_dns_records() {
        let zones = get_client()
            .await
            .get_dns_zones()
            .await
            .expect("Failed to get dns zones");
        get_client()
            .await
            .get_dns_records(&zones[0])
            .await
            .expect("Failed to get dns records");
    }

    #[test]
    async fn test_create_dns_record() {
        let zones = get_client()
            .await
            .get_dns_zones()
            .await
            .expect("Failed to get dns zones");
        let record: DnsRecord<NoId> = DnsRecord {
            id: NoId,
            host_name: "test.cs-z.hu".to_string(),
            record_type: RecordType::A,
            ttl: TTL::try_new(3600).expect("Failed to create TTL"),
            target: "8.8.8.8".to_string(),
        };
        get_client()
            .await
            .create_dns_record(&zones[0], &record)
            .await
            .expect("Failed to create dns record");
    }

    #[test]
    async fn test_update_dns_record() {
        let zones = get_client()
            .await
            .get_dns_zones()
            .await
            .expect("Failed to get dns zones");
        let records = get_client()
            .await
            .get_dns_records(&zones[0])
            .await
            .expect("Failed to get dns records");
        let mut record = records[0].clone();
        record.ttl = TTL::try_new(900).expect("Failed to create TTL");

        get_client()
            .await
            .update_dns_record(&record)
            .await
            .expect("Failed to update dns record");
    }

    #[test]
    async fn test_delete_dns_record() {
        let zones = get_client()
            .await
            .get_dns_zones()
            .await
            .expect("Failed to get dns zones");
        let records = get_client()
            .await
            .get_dns_records(&zones[0])
            .await
            .expect("Failed to get dns records");
        get_client()
            .await
            .delete_dns_record(&records[0])
            .await
            .expect("Failed to delete dns record");
    }
}
