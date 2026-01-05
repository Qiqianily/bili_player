use bili_player::fetch::{network::fetch_video_data, verify::fetch_and_verify_audio_url};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let video_data = fetch_video_data(&client, "BV1r7411p7R4").await?;
    println!("Title: {:?}", video_data);
    let audio_url =
        fetch_and_verify_audio_url(&client, &video_data.bvid, &video_data.cid.to_string()).await?;
    println!("Audio URL: {}", audio_url);
    Ok(())
}
