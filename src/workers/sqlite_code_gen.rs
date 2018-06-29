

use codegen::{Block, Function, Impl, Scope, Struct};
use models::{ColumnDef};
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

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
        db_struct.field("conn", "Connection");
        scope.push_struct(db_struct);

        scope.to_string()
    }

    fn generate_impl(name: &str, table_names: &Vec<String>)  -> String {
        let mut scope = Scope::new();
        let mut db_impl = Impl::new(name);
        let mut conn_fn = Function::new("new");
        let mut new_struct_block = Block::new("DB");
        conn_fn.vis("pub");
        conn_fn.ret(name);
        conn_fn.arg_self();
        
        new_struct_block.line("conn : Connection::open_with_flags(self.db_path, SQLITE_OPEN_READ_ONLY),");
        conn_fn.push_block(new_struct_block);
        db_impl.push_fn(conn_fn);

        for tname in table_names {
            let mut get_fn = Function::new(&format!("get_{}", tname.to_lowercase()));
            get_fn.arg_ref_self();
            // TODO: Convert this to a match when I'm done with the POC
            get_fn.line(&format!("let mut stmt = self.conn.prepare(\"SELECT * FROM {} LIMIT 25\").unwrap();", tname));
            get_fn.line("let result_iter = stmt.query_map(&[], |row| {");

            get_fn.line(&format!("\t{} {{", tname));
            get_fn.line("\t\t// put the struct field assignments here");
            get_fn.line("\t}");
            get_fn.line("}");
            
            db_impl.push_fn(get_fn);
        }


        scope.push_impl(db_impl);
        scope.to_string()
    }

    pub fn generate_db_layer(table_names: &Vec<String>) -> String {
        let struct_name = "DB";
        let extern_and_use_stmts = SqliteCodeGen::generate_use_and_extern_statements();
        let struct_str = SqliteCodeGen::generate_struct(struct_name);
        let impl_str = SqliteCodeGen::generate_impl(struct_name, table_names);
        format!("{}\n\n{}\n\n{}", extern_and_use_stmts, struct_str, impl_str)
    }
}

#[cfg(test)]
mod tests {
    use workers::sqlite_code_gen::SqliteCodeGen;
    use codegen::{Function, Impl, Scope, Struct};

    #[test] 
    fn generate_use_and_extern_statements() {
        let expected = "extern crate rusqlite;\n\nuse rusqlite::{Connection, OpenFlags};\n".to_string();
        let actual = SqliteCodeGen::generate_use_and_extern_statements();

        assert_eq!(actual, expected);
    }   

    #[test]
    fn generate_struct() {
        let expected = "pub struct DB {\n    conn: Connection,\n}".to_string();
        let db_struct = SqliteCodeGen::generate_struct("DB");

        assert_eq!(db_struct, expected);
    }

    #[test]
    fn generate_impl() {
        let expected = "impl DB {\n    pub fn new(self) -> DB {\n        DB {\n            conn : Connection::open_with_flags(self.db_path, SQLITE_OPEN_READ_ONLY),\n        }\n    }\n}".to_string();
        let db_struct = SqliteCodeGen::generate_impl("DB", &vec!["MyFile".to_string()]);

        assert_eq!(db_struct, expected);
    }

    #[test]
    fn generate_db_layer() {
        let expected = "extern crate rusqlite;\n\nuse rusqlite::{Connection, OpenFlags};\n\n\npub struct DB {\n    conn: Connection,\n}\n\nimpl DB {\n    pub fn new(self) -> DB {\n        DB {\n            conn : Connection::open_with_flags(self.db_path, SQLITE_OPEN_READ_ONLY),\n        }\n    }\n}".to_string();
        let db_struct = SqliteCodeGen::generate_db_layer(&vec!["MyFile".to_string()]);

        assert_eq!(db_struct, expected);
    }
}