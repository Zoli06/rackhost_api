use crate::auth::{AuthState, NotAuthed};
use reqwest::Url;
use reqwest_middleware::ClientWithMiddleware;
use std::marker::PhantomData;

pub(crate) static BASE_URL: once_cell::sync::Lazy<Url> = once_cell::sync::Lazy::new(|| {
    Url::parse("https://www.rackhost.hu").expect("Failed to parse URL")
});

#[derive(Debug, Clone)]
pub struct RackhostClient<A: AuthState> {
    pub(crate) _phantom_state: PhantomData<A>,
    pub(crate) client: ClientWithMiddleware,
}

impl Default for RackhostClient<NotAuthed> {
    fn default() -> Self {
        self::RackhostClient::builder().build()
    }
}
