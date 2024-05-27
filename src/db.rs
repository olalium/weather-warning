use std::error::Error;

use log::error;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLocation {
    pub id: i64,
    pub uuid: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Observation {
    pub epoch_ns: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub peak_current: i16,
    pub cloud_indicator: bool,
    pub distance_m: i64,
    pub location_id: i64,
}

pub struct Database {
    pub client: Postgrest,
}

impl Database {
    pub fn init(supabase_url: &str, supabase_api: &str) -> Database {
        return Database {
            client: Postgrest::new(supabase_url).insert_header("apiKey", supabase_api.clone()),
        };
    }

    pub async fn insert_observations(
        &self,
        observations: Vec<Observation>,
    ) -> Result<(), Box<dyn Error>> {
        let json_observations = match serde_json::to_string(&observations) {
            Ok(json_str) => json_str,
            Err(err) => {
                error!("Unable to serialize observations: {}", err);
                return Err(err.into());
            }
        };

        let response = self
            .client
            .from("observations")
            .insert(&json_observations)
            .execute()
            .await;

        match response {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Unable to write observations to db: {}", err);
                return Err(err.into());
            }
        }
    }

    pub async fn get_locations(&self) -> Result<Vec<UserLocation>, Box<dyn Error>> {
        let response_result = self.client.from("locations").select('*').execute().await;

        let response = match response_result {
            Ok(res) => res,
            Err(err) => {
                error!("Unable to get locations: {}", err);
                return Err(err.into());
            }
        };

        let response_text = match response.text().await {
            Ok(text) => text,
            Err(err) => {
                error!("Unable to get response string: {}", err);
                return Err(err.into());
            }
        };

        let locations_result: Result<Vec<UserLocation>, serde_json::Error> =
            serde_json::from_str(&response_text);

        match locations_result {
            Ok(locations) => Ok(locations),
            Err(err) => {
                error!("Unable to deserialize locations: {}", err);
                return Err(err.into());
            }
        }
    }
}
