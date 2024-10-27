use std::fmt;
use std::error;
use std::error::Error;

use crate::ualf::UalfData;
use reqwest::Client;

#[derive(Debug)]
pub enum FrostError {
    RequestError(reqwest::Error),
    ApiError(String),
}

impl fmt::Display for FrostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result  {
        match self {
            FrostError::RequestError(e) => write!(f, "HTTP request failed: {}", e),
            FrostError::ApiError(msg) => write!(f, "API error response: {}", msg)
        }
    }
}

impl error::Error for FrostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrostError::RequestError(e) => Some(e),
            FrostError::ApiError(_) => None,
        }
    }
}

impl From<reqwest::Error> for FrostError {
    fn from(err: reqwest::Error) -> FrostError {
        FrostError::RequestError(err)
    }
}


async fn get_observations(
    frost_client: &str, 
    frost_secret: &str, 
    max_age: &str
) -> Result<Vec<UalfData>, FrostError> {
    let client = Client::new();

    let response = client
        .get("https://frost.met.no/lightning/v0.ualf")
        .query(&[("referencetime", "latest"), ("maxage", max_age)])
        .basic_auth(frost_client, Some(frost_secret))
        .send()
        .await?;
    let ualf_text_data = response.text().await?;

    if ualf_text_data.starts_with("{") {
        return Err(FrostError::ApiError(ualf_text_data));
    }

    let observations: Vec<UalfData> = ualf_text_data
        .split("\n")
        .filter(|obs| !obs.is_empty())
        .filter_map(UalfData::from_string)
        .collect();

    Ok(observations)
}

pub async fn get_latest_10m_observations(
    frost_client: &str, 
    frost_secret: &str
) -> Result<Vec<UalfData>, FrostError> {
    get_observations(frost_client, frost_secret, "PT10M").await
}

pub async fn get_latest_1h_observations(
    frost_client: &str, 
    frost_secret: &str
) -> Result<Vec<UalfData>, FrostError> {
    get_observations(frost_client, frost_secret, "PT1H").await
}