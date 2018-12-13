use csv_converter::models::InputSource;
/// ConfigService is used to encapsulate the input from the user
pub trait ConfigService {
    /// Returns a Vec<InputSource> that represents all input files/sources
    fn get_input_sources(&self) -> Vec<InputSource>;
    /// Returns true if the input files have column headers, currently
    /// all files have them or none of them do
    fn has_headers(&self) -> bool;
    /// Returns true if tables/collections should be removed before
    /// loading the data
    fn should_drop_store(&self) -> bool;
}