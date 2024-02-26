pub struct Input {
    pub title: String,
    pub tipe: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Movie {
    pub channel_id: u64,
    pub title: String,
    pub total_episodes: Option<String>,
}

impl std::fmt::Display for Movie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title.trim())?;
        if let Some(total_episodes) = &self.total_episodes {
            write!(f, " ({} eps)", total_episodes)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Episode {
    pub category_id: u64,
    pub channel_id: u64,
    pub title: String,
    pub is_anime: bool,
}

impl std::fmt::Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title.trim())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Download {
    pub url: String,
    pub title: String,
}

impl std::fmt::Display for Download {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title.trim())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub thumb_url: Option<String>,
    pub data: Vec<(String, String)>,
}
