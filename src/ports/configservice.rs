use super::inputservice::InputSource;

pub trait ConfigService {
    fn get_input_sources(&self) -> Vec<InputSource>;
}