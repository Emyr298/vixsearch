use std::cmp::Ordering;

#[derive(Clone)]
pub struct Point {
    pub id: String,
    pub pos: Vec<f64>,
}

impl Point {
    pub fn new(id: String, pos: Vec<f64>) -> Self {
        Self { id, pos }
    }
}

pub(super) struct PointWithDistance<'a> {
    pub point: &'a Point,
    pub distance: f64,
}

impl<'a> PointWithDistance<'a> {
    pub fn new(point: &'a Point, distance: f64) -> Self {
        Self { point, distance }
    }
}

impl<'a> Ord for PointWithDistance<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

impl<'a> PartialOrd for PointWithDistance<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for PointWithDistance<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> Eq for PointWithDistance<'a> {}
