use reqwest::Client;
use serde_json::Value;

use crate::errors::ApplicationError;

const BASE_FETCH_AUDIO_API_URL: &str = "https://api.bilibili.com/x/player/playurl?fnval=16";
const BASE_FETCH_VIDEO_API_URL: &str = "https://api.bilibili.com/x/web-interface/view";

/// 获取音频URL
pub async fn fetch_audio_url(
    client: &Client,
    bvid: &str,
    cid: &str,
) -> Result<String, ApplicationError> {
    let url = format!("{}&bvid={}&cid={}", BASE_FETCH_AUDIO_API_URL, bvid, cid);
    tracing::info!("Fetching audio URL...");
    let response = client.get(&url).send().await?;
    let json: Value = response.json().await?;
    json["data"]["dash"]["audio"][0]["baseUrl"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ApplicationError::DataParsingError("解析音频URL失败".to_string()))
}

#[derive(serde::Deserialize, Debug)]
pub struct Owner {
    pub name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct VideoData {
    pub bvid: String,
    pub title: String,
    pub cid: i64,
    pub owner: Owner,
}

#[derive(serde::Deserialize, Debug)]
struct ApiResponse<T> {
    data: T,
}
/// 请求视频信息，获取相关数据
pub async fn fetch_video_data(client: &Client, bvid: &str) -> Result<VideoData, ApplicationError> {
    let url = format!("{}?bvid={}", BASE_FETCH_VIDEO_API_URL, bvid);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApplicationError::FetchError(format!("Fetch video data failed:{e}")))?;
    let mut api_response: ApiResponse<VideoData> = response
        .json()
        .await
        .map_err(|e| ApplicationError::FetchError(format!("Fetch video data failed:{e}")))?;
    api_response.data.bvid = bvid.to_string();
    Ok(api_response.data)
}
