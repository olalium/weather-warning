use geoutils::Location;
use std::collections::{HashMap, HashSet};

use crate::{convex_hull::compute_convex_hull, ualf::UalfData};

#[derive(Debug)]
pub struct DbscanCluster {
    pub points: Vec<UalfData>,
    pub cluster_id: usize,
}

pub struct DbscanParams {
    pub eps_km: f64,       // Maximum distance between points in kilometers
    pub min_points: usize, // Minimum points to form a cluster
}

impl Default for DbscanParams {
    fn default() -> Self {
        DbscanParams {
            eps_km: 10.0,
            min_points: 3,
        }
    }
}

pub fn cluster_lightning(data: &[UalfData], params: &DbscanParams) -> Vec<DbscanCluster> {
    let mut clusters: Vec<DbscanCluster> = Vec::new();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut point_to_cluster: HashMap<usize, usize> = HashMap::new();
    let mut current_cluster_id = 0;

    // Find neighbors within eps_km radius
    fn get_neighbors(
        point_idx: usize,
        data: &[UalfData],
        eps_km: f64,
        visited: &HashSet<usize>,
    ) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let point = &data[point_idx];

        for (idx, other) in data.iter().enumerate() {
            if visited.contains(&idx) {
                continue;
            }
            let point_loc = Location::new(point.latitude, point.longitude);
            let other_loc = Location::new(other.latitude, other.longitude);
            let distance = point_loc.haversine_distance_to(&other_loc);

            if (distance.meters() / 1000.0) <= eps_km {
                neighbors.push(idx);
            }
        }

        neighbors
    }

    // Main DBSCAN algorithm
    for point_idx in 0..data.len() {
        if visited.contains(&point_idx) {
            continue;
        }

        visited.insert(point_idx);
        let neighbors = get_neighbors(point_idx, data, params.eps_km, &visited);

        if neighbors.len() >= params.min_points {
            // Start a new cluster
            let mut cluster = DbscanCluster {
                points: vec![data[point_idx].clone()],
                cluster_id: current_cluster_id,
            };
            point_to_cluster.insert(point_idx, current_cluster_id);

            // Process neighbors
            let mut neighbor_queue = neighbors;
            while let Some(neighbor_idx) = neighbor_queue.pop() {
                if !visited.contains(&neighbor_idx) {
                    visited.insert(neighbor_idx);
                    let new_neighbors = get_neighbors(neighbor_idx, data, params.eps_km, &visited);

                    if new_neighbors.len() >= params.min_points {
                        neighbor_queue.extend(new_neighbors);
                    }
                }

                if !point_to_cluster.contains_key(&neighbor_idx) {
                    cluster.points.push(data[neighbor_idx].clone());
                    point_to_cluster.insert(neighbor_idx, current_cluster_id);
                }
            }

            clusters.push(cluster);
            current_cluster_id += 1;
        }
    }

    clusters
}

// Example usage and helper functions
impl DbscanCluster {
    pub fn center(&self) -> (f64, f64) {
        let count = self.points.len() as f64;
        let sum = self.points.iter().fold((0.0, 0.0), |acc, p| {
            (acc.0 + p.latitude, acc.1 + p.longitude)
        });
        (sum.0 / count, sum.1 / count)
    }

    pub fn average_current(&self) -> f64 {
        let sum: i32 = self.points.iter().map(|p| p.peak_current as i32).sum();
        sum as f64 / self.points.len() as f64
    }

    pub fn time_span_ns(&self) -> i64 {
        if self.points.is_empty() {
            return 0;
        }
        let min_time = self.points.iter().map(|p| p.epoch_ns).min().unwrap();
        let max_time = self.points.iter().map(|p| p.epoch_ns).max().unwrap();
        max_time - min_time
    }

    pub fn convex_hull_geo_json(&self) -> String {
        if self.points.is_empty() {
            return format!("[]");
        }

        let mut first = true;
        let mut json = format!("[");
        let convex_hull = compute_convex_hull(self.points.clone());
        for (latitude, longitude) in convex_hull {
            if first {
                json.push_str(format!("[{},{}]", latitude, longitude).as_str());
                first = false;
            } else {
                json.push_str(format!(",[{},{}]", latitude, longitude).as_str());
            }
        }
        json.push(']');
        json
    }
}
