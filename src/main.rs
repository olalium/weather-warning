use dotenv::dotenv;
use lightning_warning::{
    db::{get_locations, insert_observations},
    frost::get_latest_observations,
    location_utils::point_is_within_radius,
};
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let frost_client = std::env::var("FROST_API_CLIENT").expect("FROST_API_CLIENT must be set.");
    let frost_secret = std::env::var("FROST_API_SECRET").expect("FROST_API_SECRET must be set.");
    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
    let supabase_api =
        std::env::var("SUPABASE_API_PUBLIC").expect("SUPABASE_API_PUBLIC must be set.");

    println!("getting latest observations");
    let observations = get_latest_observations(&frost_client, &frost_secret).await;
    let locations = get_locations(&supabase_url, &supabase_api).await;

    let mut observations_within_radius = vec![];
    for observation in observations {
        if point_is_within_radius(
            locations[0].latitude,
            locations[0].longitude,
            observation.latitude,
            observation.longitude,
            &locations[0].radius_km,
        ) {
            observations_within_radius.push(observation);
        }
    }

    println!(
        "inserting {} observations to db",
        observations_within_radius.len()
    );
    insert_observations(&supabase_url, &supabase_api, observations_within_radius).await;

    Ok(())
}
