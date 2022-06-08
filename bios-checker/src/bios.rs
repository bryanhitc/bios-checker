use serde::Deserialize;
use tracing::info;

const URL: &str =
    "https://rog.asus.com/support/webapi/product/GetPDBIOS?website=us&model=ROG-STRIX-B450-I-GAMING&pdid=10277&cpu=&LevelTagId=5931";

pub async fn get_latest_version() -> anyhow::Result<u32> {
    info!("Retrieving latest BIOS version...");

    let json = reqwest::get(URL).await?.text().await?;
    let response = serde_json::from_str::<Response>(json.as_str())?;

    let version = response
        .result
        .obj
        .get(0)
        .and_then(|obj| obj.files.get(0).map(|file| file.version.as_str()))
        .and_then(|version| version.parse::<u32>().ok())
        .ok_or_else(|| anyhow::anyhow!("Unable to parse latest version"))?;

    info!("Retrieved latest BIOS version: {version}");
    Ok(version)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub result: Result,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Result {
    pub count: i64,
    pub obj: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub name: String,
    pub count: i64,
    pub files: Vec<File>,
    pub is_desc_show: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub file_size: String,
    pub release_date: String,
    pub is_release: String,
    pub pos_type: ::serde_json::Value,
    pub download_url: DownloadUrl,
    pub hardware_info_list: ::serde_json::Value,
    #[serde(rename = "INFDate")]
    pub infdate: ::serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadUrl {
    pub global: String,
    pub china: ::serde_json::Value,
}
