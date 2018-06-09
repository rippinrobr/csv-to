use std::{
    fmt,
};

use csv::StringRecord;

#[derive(PartialEq,Clone, Copy)]
pub enum DataTypes {
    EMPTY,
    F64,
    I64, 
    STRING,
}

impl DataTypes {
    pub fn string(&self) -> &str {
        match *self {
            DataTypes::EMPTY => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::STRING => "string"
        }
    }
}

impl fmt::Debug for DataTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            DataTypes::EMPTY => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::STRING => "string"
        };
        write!(f, "{:#?}", printable)
    }
}

pub struct ColumnDef{
    pub name: String, 
    pub data_type: DataTypes,
}

impl ColumnDef {
    pub fn new(name: String, data_type: DataTypes) -> ColumnDef {
        ColumnDef{
            name: name, 
            data_type: data_type,
        }
    }
}

impl fmt::Debug for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:?}", self.name, self.data_type)
    }
}

