use std::error::Error;

use log::{error, info};
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};

use crate::dbscan::DbscanCluster;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Prediction {
    pub id: i64,
    pub created_at: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PredictionInput{}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterLocationInput {
    pub prediction_id: i64,
    pub location: String // JSONB
}

pub struct Database {
    pub client: Postgrest,
    base_url: String,
    api_key: String,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            client: Postgrest::new(&self.base_url).insert_header("apiKey", &self.api_key),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

impl Database {
    pub fn init(supabase_url: &str, supabase_api: &str) -> Database {
        return Database {
            client: Postgrest::new(supabase_url).insert_header("apiKey", supabase_api),
            base_url: supabase_url.to_string(),
            api_key: supabase_api.to_string(),
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

    pub async fn insert_prediction_and_remove_old(&self, clusters: Vec<DbscanCluster>) -> Result<(), Box<dyn Error>> {
        let new_prediction = PredictionInput {};
        let json_new_prediction = serde_json::to_string(&new_prediction).unwrap();
        
        let prediction_result = self
            .client
            .from("predictions")
            .insert(&json_new_prediction)
            .single()
            .execute()
            .await;
        
        let response = match prediction_result {
            Ok(res) => res,
            Err(err) => {
                error!("Unable to create prediction: {}", err);
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

        let prediction_result: Result<Prediction, serde_json::Error> =
            serde_json::from_str(&response_text);

        let prediction = match prediction_result {
            Ok(prediction) => prediction,
            Err(err) => {
                info!("prediction result text is; {}", response_text);
                error!("Unable to deserialize prediction: {}", err);
                return Err(err.into());
            }
        };


        let mut cluster_locations: Vec<ClusterLocationInput> = vec![];
        for cluster in clusters {
            cluster_locations.push( ClusterLocationInput {
                prediction_id: prediction.id,
                location: cluster.convex_hull_geo_json()
            });
        }

        let json_cluster_locations = match serde_json::to_string(&cluster_locations) {
            Ok(json_str) => json_str,
            Err(err) => {
                error!("Unable to serialize cluster locations: {}", err);
                return Err(err.into());
            }
        };

        let response = self
            .client
            .from("cluster_locations")
            .insert(&json_cluster_locations)
            .execute()
            .await;

        match response {
            Ok(_) => (),
            Err(err) => {
                error!("Unable to write cluster locations to db: {}", err);
                return Err(err.into());
            }
        }

        let response = self
            .client
            .from("predictions")
            .not("eq", "id", prediction.id.to_string())
            .delete()
            .execute()
            .await;

        match response {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Unable to remove old predictions: {}", err);
                return Err(err.into());
            }
        }
    }
}
