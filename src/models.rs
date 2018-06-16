use std::{
    fmt,
};

use csv::StringRecord;
use barrel::Type;

#[derive(PartialEq,Clone, Copy)]
pub enum DataTypes {
    Empty,
    F64,
    I64, 
    String,
}

impl DataTypes {
    pub fn string(&self) -> &str {
        match *self {
            DataTypes::Empty => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::String => "String"
        }
    }

    pub fn to_database_type(&self) -> Type {
        match *self {
            DataTypes::Empty => Type::Text,
            DataTypes::F64 => Type::Double,
            DataTypes::I64 => Type::Integer,
            DataTypes::String => Type::Text
        }
    }
}

impl fmt::Debug for DataTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            DataTypes::Empty => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::String => "string"
        };
        write!(f, "{:#?}", printable)
    }
}

#[derive(Clone)]
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

