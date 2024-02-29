use colored::Colorize;
use humansize::{format_size, BINARY};
use reqwest::{header::CONTENT_LENGTH, Client};

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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

pub fn get_iframe_src(html: &str, index: usize) -> Option<String> {
    let re = regex!(r#"<iframe[^>]+src="([^"]+)"[^>]*>"#);
    let mut caps = re.captures_iter(html).map(|cap| cap[1].to_string());
    caps.nth(index)
}

pub async fn get_real_url(client: &Client, url: String) -> anyhow::Result<String> {
    if let Some(r) = url.split("url=").nth(1) {
        let ru = r.split("&index=").collect::<Vec<&str>>();
        let resp = client.get(ru[0]).send().await?;
        let bytes = resp.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);

        if let Some(src) = get_iframe_src(&body, ru[1].parse::<usize>()? - 1) {
            return Ok(src);
        }
    }
    Ok(url)
}

pub async fn show_image_thumb(url: String) {
    let client = Client::new();
    if let Ok(resp) = client.get(url).send().await {
        if let Ok(bytes) = resp.bytes().await {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let conf = viuer::Config {
                    transparent: true,
                    width: Some(50),
                    height: Some(30),
                    y: 8,
                    x: 2,
                    ..Default::default()
                };
                if viuer::print(&img, &conf).is_ok() {
                    println!();
                }
            }
        }
    }
}

pub fn clearscreen_and_show_banner() -> anyhow::Result<()> {
    clearscreen::clear()?;

    eprintln!(
        "{} v{} {} val \n",
        r#"
  ▄▀█ █▄░█ █ █▀▄▀█ █▀▀ █▄▀ █░█ ▄▄ █▀▀ █░░ █
  █▀█ █░▀█ █ █░▀░█ ██▄ █░█ █▄█ ░░ █▄▄ █▄▄ █ "#
            .bright_green(),
        env!("CARGO_PKG_VERSION"),
        "©".cyan()
    );
    Ok(())
}
