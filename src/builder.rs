use crate::auth::NotAuthed;
use crate::base::RackhostClient;
use async_trait::async_trait;
use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use reqwest_ratelimit::RateLimiter;
use std::marker::PhantomData;
use std::time::Duration;

pub struct MyRateLimiter {
    rate_limit: Duration,
}

#[async_trait]
impl RateLimiter for MyRateLimiter {
    async fn acquire_permit(&self) {
        tokio::time::sleep(self.rate_limit).await;
    }
}

pub struct RackhostClientBuilder<R: RateLimiter> {
    reqwest_client_builder: ClientBuilder,
    rate_limiter: R,
}

impl<R: RateLimiter> RackhostClientBuilder<R> {
    pub fn new() -> RackhostClientBuilder<MyRateLimiter> {
        let rate_limiter = MyRateLimiter {
            rate_limit: Duration::from_secs(1),
        };

        RackhostClientBuilder {
            reqwest_client_builder: ClientBuilder::new(),
            rate_limiter,
        }
    }

    pub fn client_builder(self, client_builder: ClientBuilder) -> RackhostClientBuilder<R> {
        RackhostClientBuilder {
            reqwest_client_builder: client_builder,
            rate_limiter: self.rate_limiter,
        }
    }

    pub fn rate_limit_from_duration(
        self,
        rate_limit: Duration,
    ) -> RackhostClientBuilder<MyRateLimiter> {
        let rate_limiter = MyRateLimiter { rate_limit };

        RackhostClientBuilder {
            reqwest_client_builder: self.reqwest_client_builder,
            rate_limiter,
        }
    }

    pub fn rate_limit_from_rate_limiter(
        self,
        rate_limiter: impl RateLimiter,
    ) -> RackhostClientBuilder<impl RateLimiter> {
        RackhostClientBuilder {
            reqwest_client_builder: self.reqwest_client_builder,
            rate_limiter,
        }
    }

    pub fn build(self) -> RackhostClient<NotAuthed> {
        let client = self
            .reqwest_client_builder
            .cookie_store(true)
            .redirect(Policy::none())
            // Workaround for https://github.com/hyperium/hyper/issues/2312
            .pool_max_idle_per_host(0)
            .build()
            .expect("Failed to create client");

        let client = reqwest_middleware::ClientBuilder::new(client)
            .with(reqwest_ratelimit::all(self.rate_limiter))
            .build();

        RackhostClient {
            _phantom_state: PhantomData,
            client,
        }
    }
}

impl RackhostClient<NotAuthed> {
    pub fn builder() -> RackhostClientBuilder<MyRateLimiter> {
        RackhostClientBuilder::<MyRateLimiter>::new()
    }
}
