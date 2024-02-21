use std::time::Duration;

use humansize::{format_size, BINARY};
use reqwest::{header::CONTENT_LENGTH, Client};

pub async fn get_filesize(client: &Client, url: &str) -> Option<String> {
    if url.contains("nontonanime") {
        return None;
    }
    if let Ok(resp) = client.head(url).send().await {
        let header = resp.headers();
        if let Some(content_length) = header.get(CONTENT_LENGTH) {
            if let Ok(s) = content_length.to_str() {
                if let Ok(size) = s.parse::<u64>() {
                    return Some(format_size(size, BINARY));
                }
            }
        }
    }
    None
}

pub async fn show_image_thumb(client: &Client, url: &str) -> anyhow::Result<()> {
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(2))
        .send()
        .await?;
    let bytes = resp.bytes().await?;

    let img = image::load_from_memory(&bytes)?;
    let conf = viuer::Config {
        transparent: true,
        width: Some(50),
        height: Some(30),
        y: 12,
        x: 2,
        ..Default::default()
    };
    viuer::print(&img, &conf)?;
    println!();
    Ok(())
}

pub fn get_iframe_src(html: &str, index: usize) -> Option<String> {
    let re = regex::Regex::new(r#"<iframe[^>]+src="([^"]+)"[^>]*>"#).unwrap();
    let mut caps = re.captures_iter(html).map(|cap| cap[1].to_string());
    caps.nth(index)
}

pub async fn get_real_url(client: &Client, url: String) -> anyhow::Result<String> {
    if let Some(r) = url.split("url=").nth(1) {
        let ru = r.split("&index=").collect::<Vec<&str>>();
        let resp = client
            .get(ru[0])
            .timeout(Duration::from_secs(2))
            .send()
            .await?;
        let bytes = resp.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);

        if let Some(src) = get_iframe_src(&body, ru[1].parse::<usize>()? - 1) {
            return Ok(src);
        }
    }
    Ok(url)
}
