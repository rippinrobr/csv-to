use codegen::{Function, Impl, Scope, Struct, Type};
use models::{ColumnDef};


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
            my_model.field(&c.name, c.data_type.string());
        }
        
        scope.push_struct(my_model);
        scope.to_string()
    }
}