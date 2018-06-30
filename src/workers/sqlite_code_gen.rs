

use codegen::{Block, Function, Impl, Scope, Struct};
use models::{ColumnDef};

// TODO: Swap the db connection creation in the main fn to use 
// rusqlite so that I'm using the same crate everywhere
pub struct SqliteCodeGen {
    db_path : String,
}

impl SqliteCodeGen {
    fn generate_use_and_extern_statements() -> String {
        let mut scope = Scope::new();
        
        scope.raw("extern crate rusqlite;\n");

        let mut import_scope = Scope::new(); // this is because codegen adds use statements at the top of the scope
        for use_stmt in vec![("rusqlite", "Connection"), ("rusqlite", "OpenFlags")] {
            import_scope.import(use_stmt.0, use_stmt.1);
        }

        scope.to_string() + "\n" +  &import_scope.to_string()
    }

    fn generate_struct(name: &str) -> String {
        let mut scope = Scope::new();
        let mut db_struct = Struct::new(name);
        db_struct.vis("pub");
        scope.push_struct(db_struct);
        scope.to_string()
    }

    fn generate_impl(name: &str, struct_meta: &Vec<(String, Vec<ColumnDef>)>)  -> String {
        let mut scope = Scope::new();
        let mut db_impl = Impl::new(name);
        for (tname, columns) in struct_meta {
            let mut get_fn = Function::new(&format!("get_{}", tname.to_lowercase()));
            get_fn.arg("conn", "Connection");
            get_fn.arg("page_num", "u32");
            get_fn.ret(&format!("Result<Vec<models::{}>, Error>", tname));

            // TODO: Convert this to a match when I'm done with the POC
            get_fn.line(&format!("let mut stmt = conn.prepare(\"SELECT * FROM {} LIMIT 25\").unwrap();", tname));
            get_fn.line("let result_iter = stmt.query_map(&[], |row| {");

            get_fn.line(&format!("\t{} {{", tname));
            let mut idx = 0;
            for col in columns {
                get_fn.line(&format!("\t\t{}: row.get({}),", col.name.to_lowercase(), idx));
                idx += 1;
            }
            get_fn.line("\t}");
            get_fn.line("}).unwrap();\n");
            get_fn.line("Ok(result_iter.collect())");
            
            db_impl.push_fn(get_fn);
        }


        scope.push_impl(db_impl);
        scope.to_string()
    }

    pub fn generate_db_layer(struct_meta: &Vec<(String, Vec<ColumnDef>)>) -> String {
        let struct_name = "DB";
        let extern_and_use_stmts = SqliteCodeGen::generate_use_and_extern_statements();
        let struct_str = SqliteCodeGen::generate_struct(struct_name);
        let impl_str = SqliteCodeGen::generate_impl(struct_name, struct_meta);
        format!("{}\n\n{}\n\n{}", extern_and_use_stmts, struct_str, impl_str)
    }
}

#[cfg(test)]
mod tests {
    use workers::sqlite_code_gen::SqliteCodeGen;
    use codegen::{Function, Impl, Scope, Struct};
    use models::{ColumnDef, DataTypes};

    #[test] 
    fn generate_use_and_extern_statements() {
        let expected = "extern crate rusqlite;\n\nuse rusqlite::{Connection, OpenFlags};\n".to_string();
        let actual = SqliteCodeGen::generate_use_and_extern_statements();

        assert_eq!(actual, expected);
    }   

    #[test]
    fn generate_struct() {
        let expected = "pub struct DB;".to_string();
        let db_struct = SqliteCodeGen::generate_struct("DB");

        assert_eq!(db_struct, expected);
    }

    #[test]
    fn generate_impl() {
        let col_def = ColumnDef::new("my_col".to_string(), DataTypes::String);
        let expected = 356;
        let db_struct = SqliteCodeGen::generate_impl("DB", &vec![("my_col".to_string(), vec![col_def])]);

        assert_eq!(db_struct.len(), expected);
    }

    #[test]
    fn generate_db_layer() {
        let col_def = ColumnDef::new("my_col".to_string(), DataTypes::String);
        let expected_len = 437;
        let db_struct = SqliteCodeGen::generate_db_layer(&vec![("my_col".to_string(), vec![col_def])]);

        assert_eq!(db_struct.len(), expected_len);
    }
}