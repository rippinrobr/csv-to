use super::models;

pub fn generate_struct(name: &str, headers: Vec<String>, data_types: Vec<models::DataTypes>) -> String {
    let struct_def_str = format!("pub struct {} {{", name);

    struct_def_str.to_owned()
}