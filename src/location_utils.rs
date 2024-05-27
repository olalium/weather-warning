use geoutils::{Distance, Location};
use log::warn;

use crate::{
    db::{Observation, UserLocation},
    ualf::UalfData,
};

pub fn get_observation_within_radius(
    ualf_observation: &UalfData,
    user_location: &UserLocation,
) -> Option<Observation> {
    let observation_loc = Location::new(ualf_observation.latitude, ualf_observation.longitude);
    let user_location_loc = Location::new(user_location.latitude, user_location.longitude);
    let user_radius = Distance::from_meters(user_location.radius_km * 1000);

    let distance = match observation_loc.distance_to(&user_location_loc) {
        Ok(distance) => distance,
        Err(err) => {
            warn!("Unable to find distance: {}", err);
            return None;
        }
    };

    if distance.meters() < user_radius.meters() {
        return Some(Observation {
            location_id: user_location.id,
            latitude: ualf_observation.latitude,
            longitude: ualf_observation.longitude,
            cloud_indicator: ualf_observation.cloud_indicator,
            distance_m: distance.meters() as i64,
            peak_current: ualf_observation.peak_current,
            epoch_ns: ualf_observation.epoch_ns,
        });
    }
    return None;
}
