use crate::ualf::UalfData;

const PROCESSED_BUFFER_SIZE: usize = 8192;

pub struct UalfBuffer {
    pub processed_observations: [i64; PROCESSED_BUFFER_SIZE],
    pub processed_observation_index: usize,
}

impl UalfBuffer {
    pub fn new() -> UalfBuffer {
        return UalfBuffer {
            processed_observations: [0; PROCESSED_BUFFER_SIZE],
            processed_observation_index: 0,
        };
    }

    pub fn get_unchecked_observations(&mut self, observations: &Vec<UalfData>) -> Vec<UalfData> {
        let mut unchecked_observations: Vec<UalfData> = vec![];

        for obs in observations {
            if !self.processed_observations.contains(&obs.epoch_ns) {
                unchecked_observations.push(obs.clone())
            }
        }
        if self.processed_observation_index >= PROCESSED_BUFFER_SIZE {
            self.processed_observation_index = 0
        }
        for obs in &unchecked_observations {
            self.processed_observations[self.processed_observation_index % PROCESSED_BUFFER_SIZE] =
                obs.epoch_ns;
            self.processed_observation_index += 1;
        }

        return unchecked_observations;
    }
}
