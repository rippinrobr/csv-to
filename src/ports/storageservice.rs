use csv::StringRecord;
use failure::Error;
use csv_converter::models::ColumnDef;

pub trait StorageService {
    /// describes a method that will create a table for relational databases or the equivalent in a
    /// store that is supported
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>, drop_tables: bool) -> Result<(), Error>;
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self,name: String, data: Vec<StringRecord> ) -> Result<usize, Error>;
    /// validates the number of records that existed in the CSV file were added to the store
    /// returns the true if the total_lines is equal to the number of records in the store
    fn validate(&self,name: String, total_lines: usize) -> Result<bool, Error>;
}