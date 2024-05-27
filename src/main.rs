use dotenv::dotenv;
use lightning_warning::{
    db::{Database, Observation},
    frost::get_latest_10m_observations,
    location_utils::get_observation_within_radius,
    ualf_buffer::UalfBuffer,
};
use log::info;
use reqwest::Error;
use std::{process, thread::sleep, time::Duration};

const POLLING_INTERVAL_SECONDS: u64 = 10;

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
    let mut buffer = UalfBuffer::new();

    loop {
        info!("getting latest 10 minutes of observations");
        let ualf_observations = get_latest_10m_observations(&frost_client, &frost_secret).await;

        let unchecked_observations = buffer.get_unchecked_observations(&ualf_observations);
        info!(
            "{} new observations ({}/{})",
            unchecked_observations.len(),
            unchecked_observations.len(),
            ualf_observations.len()
        );

        info!("Getting user locations");
        let locations = db.get_locations().await.unwrap_or(vec![]);
        info!("{} user locations found", locations.len());

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
            "{} observations within radius",
            observations_within_radius.len()
        );
        if !observations_within_radius.is_empty() {
            info!("inserting observations to db",);
            db.insert_observations(observations_within_radius)
                .await
                .unwrap_or(());
            info!("observations inserted into db")
        }
        info!("sleeping for {} seconds", POLLING_INTERVAL_SECONDS);
        sleep(Duration::from_secs(POLLING_INTERVAL_SECONDS));
    }
}
