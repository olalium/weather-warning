use crate::ualf::UalfData;
use reqwest::Client;

pub async fn get_latest_observations(frost_client: &str, frost_secret: &str) -> Vec<UalfData> {
    let client = Client::new();

    let ualf_text_data = client
        .get("https://frost.met.no/lightning/v0.ualf")
        .query(&[("referencetime", "latest"), ("maxage", "PT1M")])
        .basic_auth(frost_client, Some(frost_secret))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    if ualf_text_data.starts_with("{") {
        return vec![];
    }

    let observations: Vec<UalfData> = ualf_text_data
        .split("\n")
        .filter(|obs| !obs.is_empty())
        .filter_map(UalfData::from_string)
        .collect();

    return observations;
}
