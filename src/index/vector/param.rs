pub struct SearchParam {
    pub pos: Vec<f64>,
    pub cnt: usize,
}

impl SearchParam {
    pub fn new(pos: Vec<f64>, cnt: usize) -> Self {
        Self { pos, cnt }
    }

    pub fn distance(&self, other: &Vec<f64>) -> f64 {
        self.pos
            .iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
