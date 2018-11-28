
pub struct Input {
    location: String,
    size: usize,
}

pub trait InputService {
    fn new<T>(locations: Vec<String>) -> T;
}