use anyhow::anyhow;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    result: Result,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Result {
    obj: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    files: Vec<File>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct File {
    version: String,
}

const URL: &str =
    "https://rog.asus.com/support/webapi/product/GetPDBIOS?website=us&model=ROG-STRIX-B450-I-GAMING&pdid=10277&cpu=&LevelTagId=5931";

pub async fn get_latest_version() -> anyhow::Result<u32> {
    let json = reqwest::get(URL).await?.text().await?;
    let response = serde_json::from_str::<Response>(json.as_str())?;
    let first_response = response
        .result
        .obj
        .first()
        .ok_or_else(|| anyhow!("no results returned"))?;
    let first_file = first_response
        .files
        .first()
        .ok_or_else(|| anyhow!("no files returned"))?;
    let version = first_file
        .version
        .parse::<u32>()
        .map_err(|err| anyhow!("unable to parse latest version: {}", err))?;
    Ok(version)
}
