use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Package {
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Version")]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub results: Vec<Package>,
}

pub async fn get_pkglist(url: &str) -> Result<ApiResponse, reqwest::Error> {
    let json = reqwest::get(url).await?.json::<ApiResponse>().await?;
    Ok(json)
}
