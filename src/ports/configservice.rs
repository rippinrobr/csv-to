use csv_converter::models::InputSource;

pub trait ConfigService {
    fn get_input_sources(&self) -> Vec<InputSource>;
    fn has_headers(&self) -> bool;
    fn should_drop_store(&self) -> bool;
}