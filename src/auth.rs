use crate::base::{RackhostClient, BASE_URL};
use anyhow::bail;
use sealed::sealed;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Authed;
#[derive(Debug, Clone)]
pub struct NotAuthed;

#[sealed]
pub trait AuthState {}
#[sealed]
impl AuthState for Authed {}
#[sealed]
impl AuthState for NotAuthed {}

impl RackhostClient<NotAuthed> {
    pub async fn authenticate(
        self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> anyhow::Result<RackhostClient<Authed>> {
        let url = BASE_URL.join("/site/login")?;

        let csrf_token = self.get_csrf_token().await?;
        let response = self
            .client
            .post(url)
            .form(&[
                ("rackhost-csrf", csrf_token),
                ("LoginForm[username]", username.into()),
                ("LoginForm[password]", password.into()),
            ])
            .send()
            .await?;

        if !response.status().is_redirection() {
            bail!("Login failed");
        }

        Ok(RackhostClient {
            _phantom_state: PhantomData,
            client: self.client,
        })
    }
}
