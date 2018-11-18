

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleProfile {
    pub id: Option<String>,
    pub email: Option<String>,
    pub verified_email: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub link: Option<String>,
    pub picture: Option<String>,
    pub gender: Option<String>,
    pub locale: Option<String>,
}
