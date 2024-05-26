use crate::ualf::UalfData;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub id: i64,
    pub uuid: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: i16,
}

pub async fn insert_observations(
    supabase_url: &str,
    supabase_api: &str,
    observations: Vec<UalfData>,
) {
    let postgres_client =
        Postgrest::new(supabase_url).insert_header("apiKey", supabase_api.clone());

    let json_observations = serde_json::to_string(&observations).unwrap();

    postgres_client
        .from("observations")
        .insert(&json_observations)
        .execute()
        .await
        .unwrap();
}

pub async fn get_locations(supabase_url: &str, supabase_api: &str) -> Vec<Location> {
    let postgres_client =
        Postgrest::new(supabase_url).insert_header("apiKey", supabase_api.clone());
    let res = postgres_client
        .from("locations")
        .select('*')
        .execute()
        .await
        .unwrap();

    let locations: Vec<Location> = serde_json::from_str(&res.text().await.unwrap()).unwrap();

    return locations;
}
