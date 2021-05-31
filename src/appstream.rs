use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AppstreamPackage {
    #[serde(rename = "Type")]
    pub type_: String,

    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "Name")]
    pub name: HashMap<String, String>,

    #[serde(rename = "Icon")]
    pub icon: Option<Icon>,

    #[serde(rename = "Package")]
    pub package: String,

    #[serde(rename = "Summary")]
    pub summary: HashMap<String, String>,

    #[serde(rename = "Description")]
    pub description: Option<HashMap<String, String>>,

    #[serde(rename = "Categories")]
    pub categories: Option<Vec<String>>,

    #[serde(rename = "ProjectLicense")]
    pub license: Option<String>,

    #[serde(rename = "Url")]
    pub urls: Option<HashMap<String, String>>,

    #[serde(rename = "Launchable")]
    pub launchable: Option<Launchable>,

    pub origin: Option<String>,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Icon {
    cached: Option<Vec<CachedIcon>>,
    stock: Option<String>,
    remote: Option<Vec<RemoteIcon>>,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CachedIcon {
    name: String,
    width: u16,
    height: u16,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RemoteIcon {
    url: String,
    width: u16,
    height: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Launchable {
    #[serde(rename = "desktop-id")]
    desktop_id: Vec<String>
}