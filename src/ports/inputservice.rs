use csv::StringRecord;
use csv_converter::models::ColumnDef;

#[derive(Clone, Debug)]
pub struct InputSource {
    pub location: String,
    pub size: u64,
    pub columns: Vec<ColumnDef>,
    pub content: Vec<StringRecord>,
}

pub trait InputService {
    fn parse(&self) -> Vec<InputSource> ;
}