use serde::Deserialize;

const URL: &str =
    "https://rog.asus.com/support/webapi/product/GetPDBIOS?website=us&model=ROG-STRIX-B450-I-GAMING&pdid=10277&cpu=&LevelTagId=5931";

pub async fn get_latest_version() -> anyhow::Result<u32> {
    let json = reqwest::get(URL).await?.text().await?;
    let response = serde_json::from_str::<Response>(json.as_str())?;

    let version = response
        .result
        .obj
        .into_iter()
        .next()
        .and_then(|obj| obj.files.into_iter().next().map(|file| file.version))
        .and_then(|version| version.parse::<u32>().ok())
        .ok_or_else(|| anyhow::anyhow!("Unable to parse latest version"))?;

    Ok(version)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    #[serde(rename = "Result")]
    pub result: Result,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Result {
    #[serde(rename = "Count")]
    pub count: i64,
    #[serde(rename = "Obj")]
    pub obj: Vec<Item>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Count")]
    pub count: i64,
    #[serde(rename = "Files")]
    pub files: Vec<File>,
    #[serde(rename = "IsDescShow")]
    pub is_desc_show: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "FileSize")]
    pub file_size: String,
    #[serde(rename = "ReleaseDate")]
    pub release_date: String,
    #[serde(rename = "IsRelease")]
    pub is_release: String,
    #[serde(rename = "PosType")]
    pub pos_type: ::serde_json::Value,
    #[serde(rename = "DownloadUrl")]
    pub download_url: DownloadUrl,
    #[serde(rename = "HardwareInfoList")]
    pub hardware_info_list: ::serde_json::Value,
    #[serde(rename = "INFDate")]
    pub infdate: ::serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadUrl {
    #[serde(rename = "Global")]
    pub global: String,
    #[serde(rename = "China")]
    pub china: ::serde_json::Value,
}
