use std::cmp::Ordering;
use crate::ualf::UalfData;

pub fn compute_convex_hull(points: Vec<UalfData>) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.iter()
                .map(|p| (p.latitude, p.longitude))
                .collect();
    }

    // Convert to (lat, lon) pairs
    let mut points: Vec<(f64, f64)> = points.iter()
        .map(|p| (p.latitude, p.longitude))
        .collect();

    // Find the point with the lowest latitude (and leftmost if tied)
    let mut bottom_idx = 0;
    for (i, point) in points.iter().enumerate().skip(1) {
        if point.0 < points[bottom_idx].0 || 
           (point.0 == points[bottom_idx].0 && point.1 < points[bottom_idx].1) {
            bottom_idx = i;
        }
    }

    // Put the bottom point first
    points.swap(0, bottom_idx);
    let bottom_point = points[0];

    // Sort points by polar angle and distance from bottom point
    points[1..].sort_by(|a, b| {
        let angle_a = (a.0 - bottom_point.0).atan2(a.1 - bottom_point.1);
        let angle_b = (b.0 - bottom_point.0).atan2(b.1 - bottom_point.1);
        
        match angle_a.partial_cmp(&angle_b) {
            Some(Ordering::Equal) => {
                // If angles are equal, sort by distance from bottom point
                let dist_a = (a.1 - bottom_point.1).hypot(a.0 - bottom_point.0);
                let dist_b = (b.1 - bottom_point.1).hypot(b.0 - bottom_point.0);
                dist_a.partial_cmp(&dist_b).unwrap()
            },
            Some(ordering) => ordering,
            None => Ordering::Equal,
        }
    });

    // Graham's scan
    let mut hull = vec![points[0]];
    
    for point in points.iter().skip(1) {
        while hull.len() >= 2 {
            let p1 = hull[hull.len() - 2];
            let p2 = hull[hull.len() - 1];
            
            // Calculate cross product to determine turn direction
            let cross_product = (p2.1 - p1.1) * (point.0 - p1.0) -
                              (point.1 - p1.1) * (p2.0 - p1.0);
            
            // If we make a right turn or go straight, pop the last point
            if cross_product <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(*point);
    }

    hull
}