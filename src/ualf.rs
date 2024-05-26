use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UalfData {
    pub epoch_ns: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub peak_current: i16,
    pub cloud_indicator: bool,
}

impl UalfData {
    pub fn from_string(ualf_str: &str) -> Option<UalfData> {
        let split_observation: Vec<f64> = ualf_str
            .split(' ')
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();

        let year = split_observation[1] as i32;
        let month = split_observation[2] as u32;
        let day = split_observation[3] as u32;
        let hour = split_observation[4] as u32;
        let minutes = split_observation[5] as u32;
        let seconds = split_observation[6] as u32;
        let nanos = split_observation[7] as u32;

        let epoch = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_nano_opt(hour, minutes, seconds, nanos)
            .unwrap()
            .and_utc()
            .timestamp_nanos_opt()
            .unwrap();

        return Some(UalfData {
            epoch_ns: epoch,
            latitude: split_observation[8],
            longitude: split_observation[9],
            peak_current: split_observation[10] as i16,
            cloud_indicator: split_observation[21] != 0f64,
        });
    }
}
