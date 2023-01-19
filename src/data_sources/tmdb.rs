use reqwest::Client;
use serde::Deserialize;

pub struct TMDB {
    pub client: Client,
    pub token: String,
}

#[derive(Deserialize)]
pub struct TMDBSearchResult {
    pub id: Option<u32>,
    pub backdrop_path: Option<String>,
    pub poster_path: Option<String>,
    pub media_type: Option<String>,
    pub genre_ids: Option<Vec<u32>>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub origin_country: Option<Vec<u32>>,
    pub original_langauage: Option<String>,
    pub original_name: Option<String>,
    pub popularity: Option<f32>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<u32>,
    pub first_air_date: Option<String>,
    pub release_date: Option<String>,
    pub adult: Option<bool>,
    pub video: Option<bool>,
}

#[derive(Deserialize)]
pub struct TMDBSearchResponse {
    pub page: u32,
    pub results: Option<Vec<TMDBSearchResult>>,
    pub total_pages: u32,
    pub total_results: u32,
}

impl TMDB {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub async fn search(&self, query: &str) -> Option<TMDBSearchResponse> {
        let res = self
            .client
            .get("https://api.themoviedb.org/3/search/multi")
            .bearer_auth(&self.token)
            .query(&[(&"query", query)])
            .send()
            .await;
        match res {
            Ok(r) => {
                let data = r.json::<TMDBSearchResponse>().await;
                match data {
                    Ok(response) => Some(response),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}
