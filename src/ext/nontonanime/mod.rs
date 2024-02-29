use reqwest::Client;
use serde_json::Value;

use crate::{
    models::{Episode, Stream},
    util::{get_filesize, get_real_url},
};

pub mod anime;
pub mod movie;

static USER_PASS: &str = "drakornicojanuar:DIvANTArtBInsTriSkEremeNtOMICErCeSMiQUaKarypsBoari";

pub async fn get_stream_urls(client: &Client, episode: Episode) -> anyhow::Result<Vec<Stream>> {
    let url = "https://animeku.my.id/nontonanime-v77/phalcon/api/get_post_description_secure/v9_4/";
    let payload = format!("channel_id={}&isAPKvalid=true", episode.id);

    let response = client
        .post(url)
        .header("Cache-Control", "max-age=0")
        .header("Data-Agent", "New Aniplex v9.1")
        .header("Accept-Encoding", "gzip")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", payload.len())
        .header("Host", "animeku.my.id")
        .header("Connection", "Keep-Alive")
        .header("User-Agent", "okhttp/3.12.13")
        .body(payload)
        .send()
        .await?;

    let json = response.json::<Value>().await?;
    let mut streams = vec![];
    for (n, reso) in [
        ("channel_url", "360p SD"),
        ("channel_url_hd", "720p HD"),
        ("channel_url_fhd", "1080p FHD"),
    ] {
        if let Some(raw_url) = json[n].as_str() {
            let mut reso = reso.to_string();
            let mut url = raw_url.trim_matches('"').trim().to_string();
            if url.contains("whatbox") {
                if let Some(new_url) = url.strip_prefix("http://") {
                    url = format!("http://{}@{}", USER_PASS, new_url);
                }
            } else if url.contains("nontonanime") {
                if let Ok(direct_url) = get_real_url(client, url.clone()).await {
                    url = direct_url
                }
            }

            if url.starts_with("http") {
                if let Some(size) = get_filesize(client, &url).await {
                    reso.push_str(" (");
                    reso.push_str(size.as_str());
                    reso.push(')')
                }

                streams.push(Stream { url, title: reso });
            }
        }
    }
    if streams.len() == 1 && !episode.is_series {
        let new_title = streams[0].title.replace("360p SD", "720p HD");
        streams[0].title = new_title;
    }
    Ok(streams)
}
