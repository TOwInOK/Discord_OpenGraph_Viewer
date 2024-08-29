use miniserde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct FullUser {
    /// Data of user
    pub user: User,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub global_name: String,
    pub avatar: String,
    pub banner_color: Option<String>,
    pub bio: String,
}
