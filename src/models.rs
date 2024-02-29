pub struct Input {
    pub title: String,
    pub tipe: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Movie {
    pub id: String,
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
    pub id: String,
    pub title: String,
    pub is_series: bool,
}

impl std::fmt::Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title.trim())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stream {
    pub url: String,
    pub title: String,
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title.trim())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub thumb_url: Option<String>,
    pub data: Vec<(String, String)>,
}
