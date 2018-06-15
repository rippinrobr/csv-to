use codegen::{Function, Impl, Module, Scope, Struct, Type};
use models::{ColumnDef};
use csv::StringRecord;
use models::{DataTypes};
use std::io::Error;

pub struct CodeGen;

impl CodeGen {
    
    pub fn generate_struct(name: &str, columns: Vec<ColumnDef>) -> String {
        let mut scope = Scope::new();
        let mut my_model = Struct::new(name);
        
        my_model
            .derive("Debug")
            .derive("Deserialize")
            .derive("Serialize")
            .vis("pub");

        for c in columns.into_iter() {
            my_model.field(&c.name.to_lowercase(), c.data_type.string());
        }
        
        scope.push_struct(my_model);
        
        scope.to_string()
    }

    pub fn generate_mod_file_contents(model_file_names: Vec<String>) -> String{
        let mut scope = Scope::new();

        for file_name in model_file_names.iter() {
            scope.raw(&format!("pub mod {};", file_name.to_lowercase()));
        }

        scope.to_string()
    }
}