use anyhow::Result;
use axum::response::Html;
use full::FullUser;
use miniserde::{json, Serialize};
use tracing::{instrument, trace};

mod full;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    /// id
    pub id: String,
    /// part of cdn url
    pub avatar: String,
    /// custom username
    pub global_name: String,
    /// main username
    pub username: Option<String>,
    /// Main color (for non premium)
    pub banner_color: String,
    /// Second color
    // pub accent_color: u64,
    /// Custom info
    pub bio: String,
}

impl From<FullUser> for UserInfo {
    fn from(value: FullUser) -> Self {
        Self {
            id: value.user.id,
            avatar: value.user.avatar,
            global_name: value.user.global_name,
            username: value.user.username,
            banner_color: value.user.banner_color.unwrap_or("#FFFFFF".to_owned()),
            // accent_color: value.user.accent_color,
            bio: value.user.bio,
        }
    }
}

impl UserInfo {
    /// Make [UserInfo]
    #[instrument]
    pub async fn search(id: &str, token: &str) -> Result<UserInfo> {
        let url = format!("https://discord.com/api/v10/users/{}/profile", id);
        let client = reqwest::Client::new()
            .get(url)
            .header("Authorization", token)
            .send()
            .await;
        let data = client.unwrap().text().await?;
        let data: FullUser = json::from_str(&data)?;
        Ok(data.into())
    }
    /// Generate Card by [UserInfo]
    #[instrument(skip(self))]
    pub fn generate_card(&self) -> Html<String> {
        let title = if self.username.is_some()
            && self.global_name.to_lowercase() == self.username.as_ref().unwrap().to_lowercase()
            || self.username.is_none()
        {
            self.global_name.to_string()
        } else {
            format!(
                "{} aka {}",
                self.global_name,
                self.username.as_ref().unwrap()
            )
        };
        trace!("Data: {:#?}", self);
        let html_content = format!(
            r#"<!DOCTYPE html>
                   <html>
                   <head>
                       <meta property="og:title" content="{title}">
                       <meta name="theme-color" content="{banner_color}">
                       <meta property="og:url" content="https://discord.com/users/{uid}" />
                       <meta property="og:site_name" content="Discord" />
                       <meta property="og:image" content="https://cdn.discordapp.com/avatars/{uid}/{avatar}?size=1024" />
                       <meta property="og:description" content="{bio}" />
                   </head>
                   <body>
                   </body>
                   <script>
                       window.location.replace("https://discord.com/users/{uid}");
                   </script>
                   </html>"#,
            title = title,
            banner_color = self.banner_color,
            uid = self.id,
            avatar = self.avatar,
            bio = self.bio,
        );
        Html(html_content)
    }
}
