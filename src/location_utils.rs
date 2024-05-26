use geoutils::{Distance, Location};

pub fn point_is_within_radius(
    lat_center: f64,
    lon_center: f64,
    lat_obs: f64,
    lon_obs: f64,
    radius_km: &i16,
) -> bool {
    let center = Location::new(lat_center, lon_center);
    let observation = Location::new(lat_obs, lon_obs);
    let distance = Distance::from_meters(radius_km * 1000);
    return observation.is_in_circle(&center, distance).unwrap();
}
