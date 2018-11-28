
use crate::ports::inputservice::{Input, InputService};

pub struct CSVService {
    locations: Vec<String>
}

impl InputService for CSVService {
    /// Creates a new instance of the CSVService
    fn new(locations: Vec<String>) -> InputService {
        CSVService {
            locations,
        }
    }
}