
use crate::ports::inputservice::{InputSource, InputService};

#[derive(Clone,Debug)]
pub struct CSVService<'a> {
    inputs: &'a [InputSource]
}

impl<'a> CSVService<'a> {
    /// Creates a new instance of the CSVService
    pub fn new(inputs: &[InputSource]) -> CSVService {
        CSVService {
            inputs,
        }
    }
}

impl<'a> InputService for CSVService<'a> {

    fn parse(&self) -> Vec<InputSource> {
        let inputs: Vec<InputSource> = Vec::new();

        inputs.to_owned()
    }
}