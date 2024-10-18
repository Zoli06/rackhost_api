use crate::auth::{AuthState, NotAuthed};
use reqwest::{redirect::Policy, Client, ClientBuilder, Url};
use std::marker::PhantomData;

pub(crate) static BASE_URL: once_cell::sync::Lazy<Url> = once_cell::sync::Lazy::new(|| {
    Url::parse("https://www.rackhost.hu").expect("Failed to parse URL")
});

#[derive(Debug, Clone)]
pub struct RackhostClient<A: AuthState> {
    pub(crate) _phantom_state: PhantomData<A>,
    pub(crate) client: Client,
}

impl RackhostClient<NotAuthed> {
    pub fn new(client_builder: ClientBuilder) -> Self {
        let client = client_builder
            .cookie_store(true)
            .redirect(Policy::none())
            // Workaround for https://github.com/hyperium/hyper/issues/2312
            .pool_max_idle_per_host(0)
            .build()
            .expect("Failed to create client");

        Self {
            _phantom_state: PhantomData,
            client,
        }
    }
}

impl Default for RackhostClient<NotAuthed> {
    fn default() -> Self {
        Self::new(Client::builder())
    }
}
