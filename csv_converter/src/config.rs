extern crate toml;

use input::*;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DbCfg {
    pub db_type: Option<String>,
    pub db_uri: Option<String>
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CodeGenCfg {
    pub output_dir: String,
    pub project_name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct OutputCfg {
    pub output_dir: String,
    pub project_name: Option<String>
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub gen_models: Option<bool>,
    pub gen_sql: Option<bool>,
    pub gen_webserver: Option<bool>, 
    pub input_type: InputType,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub output: OutputCfg,
    pub output_db: DbCfg,
}

impl Config {

    pub fn load(config_str: &str) -> Config {
        match toml::from_str(config_str) {
            Ok(config) => {
                config
            },
            Err(e) => panic!("############################################\n{}", format!("Config ERROR: {}", e))
        }
    }

    pub fn does_project_dir_exist(&self) -> Result<bool, String>{
        let project_path = Path::new(&self.output.output_dir);

        if project_path.exists() {
            if project_path.is_dir() {
                return Ok(true);
            } 
            return Err(format!("The path '{}' provided is a file, not a directory.", self.output.output_dir).clone());  
        } 

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_input() {
        let test_yaml = r#"
        input_type = 'CSV'
        files = ['a.csv', 'b.csv' ]
        directories = ['~/src/baseballdatabank/core', '~/src/hockeydatabank' ]

        "#;
        
        let actual = Config::load(test_yaml);

        assert_eq!("CSV", actual.input_type.to_string());
        assert_eq!(2, actual.files.len());
        assert_eq!("a.csv", actual.files[0]);
        assert_eq!("b.csv", actual.files[1]);
        assert_eq!(2, actual.directories.len());
        assert_eq!("~/src/baseballdatabank/core", actual.directories[0]);
        assert_eq!("~/src/hockeydatabank", actual.directories[1]);
    }
}