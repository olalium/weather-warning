use dotenv::dotenv;
use lightning_warning::{
    db::{Database, Observation},
    frost::{get_latest_10m_observations, get_latest_1h_observations, FrostError},
    location_utils::get_observation_within_radius,
    ualf_buffer::UalfBuffer, dbscan::{cluster_lightning, DbscanParams},
};
use log::{error, info};
use reqwest::Error;
use tokio::task;
use std::{process, thread::sleep, time::{Duration, Instant}};

const POLLING_INTERVAL_SECONDS: u64 = 10;
const ERROR_INTERVAL_SECONDS: u64 = POLLING_INTERVAL_SECONDS * 5;
const PREDICITON_INTERVAL_SECONDS: u64 = 60;


async fn observation_loop(
    frost_client: String,
    frost_secret: String,
    db: Database,
) {
    let mut buffer = UalfBuffer::new();

    loop {
        info!("[OBSERVATION] getting latest 10 minutes of observations");
        let ualf_observations = match get_latest_10m_observations(&frost_client, &frost_secret).await {
            Ok(observations) => observations,
            Err(e) => {
                match e {
                    FrostError::ApiError(msg) => error!("Failed to fetch observations: {}", msg),
                    FrostError::RequestError(e) => error!("Frost API error: {}", e)
                }
                info!("sleeping for {} seconds", ERROR_INTERVAL_SECONDS);
                sleep(Duration::from_secs(ERROR_INTERVAL_SECONDS));
                continue;
            },
        };

        let unchecked_observations = buffer.get_unchecked_observations(&ualf_observations);
        info!(
            "[OBSERVATION] {} new observations ({}/{})",
            unchecked_observations.len(),
            unchecked_observations.len(),
            ualf_observations.len()
        );

        info!("[OBSERVATION] Getting user locations");
        let locations = db.get_locations().await.unwrap_or(vec![]);
        info!("[OBSERVATION] {} user locations found", locations.len());

        let mut observations_within_radius: Vec<Observation> = vec![];
        for location in &locations {
            for ualf_observation in &unchecked_observations {
                match get_observation_within_radius(ualf_observation, location) {
                    Some(ok) => observations_within_radius.push(ok),
                    None => (),
                };
            }
        }
        info!(
            "[OBSERVATION] {} observations within radius",
            observations_within_radius.len()
        );
        if !observations_within_radius.is_empty() {
            info!("[OBSERVATION] inserting observations to db",);
            db.insert_observations(observations_within_radius)
                .await
                .unwrap_or(());
            info!("[OBSERVATION] observations inserted into db")
        }
        info!("[OBSERVATION] sleeping for {} seconds", POLLING_INTERVAL_SECONDS);
        sleep(Duration::from_secs(POLLING_INTERVAL_SECONDS));
    }
}

async fn prediction_loop(
    frost_client: String,
    frost_secret: String,
    db: Database,
) {
    loop {
        info!("[PREDICTION] getting latest 1 hour of observations");
        let ualf_observations = match get_latest_1h_observations(&frost_client, &frost_secret).await {
            Ok(observations) => observations,
            Err(e) => {
                match e {
                    FrostError::ApiError(msg) => error!("Failed to fetch observations: {}", msg),
                    FrostError::RequestError(e) => error!("Frost API error: {}", e)
                }
                info!("sleeping for {} seconds", ERROR_INTERVAL_SECONDS);
                sleep(Duration::from_secs(ERROR_INTERVAL_SECONDS));
                continue;
            },
        };
        if ualf_observations.is_empty() {
            info!("No observations found the last hour");
            info!("sleeping for {} seconds", PREDICITON_INTERVAL_SECONDS*5);
            sleep(Duration::from_secs(PREDICITON_INTERVAL_SECONDS*5));
        }

        info!("[PREDICTION] Found {} observations", ualf_observations.len());
        info!("[PREDICTION] finding lightning clusters");
        let now = Instant::now();
        let clustered_observations = cluster_lightning(&ualf_observations, &DbscanParams::default());
        let elapsed = now.elapsed().as_millis();
        info!("[PREDICTION] dbscan algo took {:.2?}ms", elapsed);
        info!("[PREDICTION] Found {} clusters", clustered_observations.len());

        if !clustered_observations.is_empty() {
            db.insert_prediction(clustered_observations).await.unwrap_or(());
        }

        info!("[PREDICTION] sleeping for {} seconds", PREDICITON_INTERVAL_SECONDS);
        sleep(Duration::from_secs(PREDICITON_INTERVAL_SECONDS));
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();

    info!("My pid is {}", process::id());

    let frost_client = std::env::var("FROST_API_CLIENT").expect("FROST_API_CLIENT must be set.");
    let frost_secret = std::env::var("FROST_API_SECRET").expect("FROST_API_SECRET must be set.");
    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
    let supabase_api =
        std::env::var("SUPABASE_API_PUBLIC").expect("SUPABASE_API_PUBLIC must be set.");

    let db: Database = Database::init(&supabase_url, &supabase_api);

    let db_clone = db.clone();
    let frost_client_prediction = frost_client.clone();
    let frost_secret_prediction = frost_secret.clone();

    let observation_handle = task::spawn(observation_loop(
        frost_client,
        frost_secret,
        db,
    ));
    
    let prediction_handle = task::spawn(prediction_loop(frost_client_prediction, frost_secret_prediction, db_clone));

    tokio::try_join!(observation_handle, prediction_handle).unwrap();

    Ok(())
}
