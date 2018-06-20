use codegen::{Impl, Scope, Struct};
use models::{ColumnDef};

pub struct CodeGen;

impl CodeGen {
    
    pub fn generate_struct(name: &str, columns: &Vec<ColumnDef>) -> String {
        let mut scope = Scope::new();
        let mut my_model = Struct::new(name);
        
        if columns.len() > 0 {
            my_model
                .derive("Debug")
                .derive("Deserialize")
                .derive("Serialize");    
        }

        my_model.vis("pub");
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

    pub fn generate_db_actor() -> String {
        let mut scope = Scope::new();

        scope.import("actix::prelude", "*");
        scope.import("sqlite", "Connection");
        scope.raw("pub struct DbExecutor(pub Connection);");
        
        let mut actor_trait = Impl::new("DbExecutor");
        actor_trait.impl_trait("Actor");
        actor_trait.associate_type("Context", "SyncContext<Self>");

        scope.push_impl(actor_trait);
    
        scope.to_string()
    }
}

#[cfg(test)]
mod tests {
    use workers::code_gen::CodeGen;
    use models::{ColumnDef, DataTypes};

    #[test] 
    fn generate_struct() {
        let struct_def = "#[derive(Debug, Deserialize, Serialize)]\npub struct people {\n    name: String,\n    age: i64,\n    weight: f64,\n}".to_string();
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("name".to_string(), DataTypes::String), ColumnDef::new("age".to_string(), DataTypes::I64), ColumnDef::new("weight".to_string(), DataTypes::F64)];
        assert_eq!(struct_def, CodeGen::generate_struct("people", &cols));
    }

    #[test] 
    fn generate_struct_with_no_columns() {
        let struct_def = "pub struct people;".to_string();
        let cols: Vec<ColumnDef> = vec![];
        assert_eq!(struct_def, CodeGen::generate_struct("people", &cols));
    }

    #[test]
    fn generate_db_actor() {
        let db_actor = "".to_string();
        assert_eq!(db_actor, CodeGen::generate_db_actor());
    }
}