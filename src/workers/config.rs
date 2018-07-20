extern crate toml;

use workers::input::*;
use std::error::Error;
use std::fs::{create_dir_all};
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SqliteCfg {
    db_path: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DbCfg {
    sqlite: Option<SqliteCfg>,
}

impl DbCfg {
    pub fn has_sqlite(self) -> bool {
        if let Some(_) = self.sqlite {
            return true;
        }

        false
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CodeGenCfg {
    pub project_dir: String,
    pub project_name: String,
}

impl CodeGenCfg{
    pub fn create_project_dir(self) -> io::Result<()> {
        let src_path = &format!("{}/{}/src/models", self.project_dir, self.project_name);
        if Path::new(src_path).exists() {
            return Ok(());
        }

        create_dir_all(src_path)
    }

    pub fn models_dir(self) -> String {
        format!("{}/{}/src/models", self.project_dir, self.project_name)
    }
       
    pub fn actors_dir(self) -> String {
        format!("{}/{}/src/actors", self.project_dir, self.project_name)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct OutputCfg {
    pub db: Option<DbCfg>,
    pub code_gen: Option<CodeGenCfg>
}

impl OutputCfg {
    pub fn create_project(self) -> Result<(), String>  {
        if let Some(code_gen_cfg) = self.code_gen {
            let src_path = &format!("{}/{}/src/models", code_gen_cfg.project_dir, code_gen_cfg.project_name);
            if Path::new(src_path).exists() {
                return Ok(());
            } 
            

            let res = match create_dir_all(src_path) {
                Err(e) => Err(format!("Error: {}", e)),
                _ => Ok(())
            };

            return res;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub gen_models: Option<bool>,
    pub gen_sql: Option<bool>,
    pub gen_webserver: Option<bool>, 
    pub input_type: InputType,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub output: OutputCfg
}

impl Config {

    pub fn load(config_str: &str) -> Config {
        match toml::from_str(config_str) {
            Ok(config) => {
                config
                // let mut cfg: Config = config;
                // cfg.add_files_in_directories();
                // (cfg, cfg.add_files_in_directories())
            },
            Err(e) => panic!("############################################\n{}", format!("Config ERROR: {}", e))
        }
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