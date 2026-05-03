use crate::index::vector::entity::PointWithDistance;

use super::entity::Point;
use super::param::SearchParam;
use super::result::SearchResult;
use std::collections::BinaryHeap;
use std::io::{Error, ErrorKind};

pub struct Index {
    dimension: usize,
    points: Vec<Point>,
}

impl Index {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            points: Vec::new(),
        }
    }

    pub fn insert(&mut self, point: Point) -> Result<(), Error> {
        if point.pos.len() != self.dimension {
            return Err(Error::new(ErrorKind::InvalidInput, "Dimension mismatch"));
        }

        self.points.push(point);

        Ok(())
    }

    pub fn search(&self, param: SearchParam) -> Result<SearchResult, Error> {
        if param.pos.len() != self.dimension {
            return Err(Error::new(ErrorKind::InvalidInput, "Dimension mismatch"));
        }

        let mut max_heap: BinaryHeap<PointWithDistance> = BinaryHeap::new();

        for point in &self.points {
            let distance = param.distance(&point.pos);
            let point_dist = PointWithDistance::new(point, distance);
            max_heap.push(point_dist);

            if max_heap.len() > param.cnt {
                max_heap.pop();
            }
        }

        let points = max_heap
            .into_sorted_vec()
            .iter()
            .map(|x| x.point)
            .cloned()
            .collect();

        Ok(SearchResult::new(points))
    }
}
