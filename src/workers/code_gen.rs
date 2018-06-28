use codegen::{Function, Impl, Scope, Struct};
use models::{ColumnDef};
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

pub struct CodeGen;

impl CodeGen {
    
    pub fn generate_handler(name: &str) -> Function {
        let mut myfn = Function::new(name);
        myfn
            .arg("_req", "HttpRequest<State>")
            .ret("&'static str")
            .line(format!("\"Pretend this is a list of {}\"", name));

        myfn
    }

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

    pub fn generate_mod_file_contents(mod_names: &Vec<String>) -> String{
        let mut scope = Scope::new();

        for file_name in mod_names.iter() {
            scope.raw(&format!("pub mod {};", file_name.to_lowercase().replace(".rs", "")));
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

    pub fn generate_webservice(db_path: String, entities: &Vec<String>) -> String {
        let mut scope = Scope::new();
        
        for use_stmt in vec![("actix", "{Addr,Syn}"), ("actix::prelude", "*"), ("actors::db_actor", "*"), ("actix_web", "http, App, AsyncResponder, HttpRequest, HttpResponse"),
                            ("actix_web::server", "HttpServer"), ("futures", "Future"), ("actix_web", "Error"), ("actix_web", "Json"), ("actix_web::middleware", "Logger")] {
            scope.import(use_stmt.0, use_stmt.1);
        }
        
        let mut state_struct = Struct::new("State");
        state_struct
            .doc("This is state where we will store *DbExecutor* address.")
            .field("db", "Addr<Syn, DbExecutor>");
        scope.push_struct(state_struct);

        let mut handlers = Struct::new("RouteHandlers");
        handlers.doc("Used to implement all of the route handlers");
        scope.push_struct(handlers);

        let mut index_fn = Function::new("index");
        index_fn
                .arg("_req", "HttpRequest<State>")
                .ret("&'static str")
                .line("\"Put the next steps instructions here\"");

        let mut handler_impl = Impl::new("RouteHandlers");
        handler_impl.push_fn(index_fn);

        for ent in entities {
            // add the handler funciton creation call here
            handler_impl.push_fn(CodeGen::generate_handler(&ent.to_lowercase()));
        }
        scope.push_impl(handler_impl);
        scope.raw("");

        create_extern_create_defs() + &scope.to_string() + &create_main_fn(db_path, &entities)
    }

    pub fn write_code_to_file(dir_path: &str, file_name: &str, code: String) -> Result<String, Error> {

        match File::create(format!("{}/{}", dir_path, &file_name).to_lowercase()) {
            Ok(mut file) => {
                match file.write_all(&code.into_bytes()) {
                    Ok(_) => Ok(file_name.to_string()),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn create_curl_script(output_dir: &str, entities: &Vec<String>) -> Result<String, Error> {
        let mut scope = Scope::new();
        scope.raw("#!/bin/bash\n");
        for ent in entities {
                let lower_ent = ent.to_lowercase().replace(".rs", "");
                scope.raw(&format!("curl http://localhost:8088/{}", lower_ent));
        }

        return CodeGen::write_code_to_file(output_dir, "curl_test.sh", scope.to_string().replace("\n\n", "\n"))    
    }
}

fn create_extern_create_defs() -> String {
    let mut extern_scope = Scope::new(); 
        for extern_crate in vec!["pub mod actors;\npub mod models;\n\n\nextern crate clap;", "extern crate dotenv;", "extern crate env_logger;", "extern crate actix;", 
                                "extern crate actix_web;", "extern crate sqlite;", "extern crate futures;", "#[macro_use]", "extern crate serde_derive;"] {
            extern_scope.raw(extern_crate);
        }
        extern_scope.raw("\n");

        extern_scope.to_string().replace("\n\n", "\n")
}

fn create_main_fn(db_path: String, entities: &Vec<String>) -> String {
    let mut main_fn_scope = Scope::new();
        main_fn_scope.raw("fn main() {");
        main_fn_scope.raw("\tstd::env::set_var(\"RUST_LOG\", \"actix_web=info\");");
        main_fn_scope.raw("\tenv_logger::init();");
        main_fn_scope.raw("\tlet sys = actix::System::new(\"csv2api\");");

        main_fn_scope.raw("// Start 3 parallel db executors");
        main_fn_scope.raw("\tlet addr = SyncArbiter::start(3, || {");
        main_fn_scope.raw(&format!("\t    DbExecutor(sqlite::open(\"{}\").unwrap())", db_path));
        main_fn_scope.raw("\t});");

        main_fn_scope.raw("\tHttpServer::new(move || {");
        main_fn_scope.raw("\t\tApp::with_state(State{db: addr.clone()})");
        main_fn_scope.raw("\t\t\t.middleware(Logger::default())");
        main_fn_scope.raw("\t\t\t.resource(\"/\", |r| r.method(http::Method::GET).f(RouteHandlers::index))");

        for ent in entities {
            let lower_ent = ent.to_lowercase();
            main_fn_scope.raw(&format!("\t\t\t.resource(\"/{}\", |r| r.method(http::Method::GET).f(RouteHandlers::{}))", lower_ent, lower_ent));
        }

        main_fn_scope.raw("\t})");
        main_fn_scope.raw("\t.bind(\"127.0.0.1:8088\").unwrap()");
        main_fn_scope.raw("\t.start();\n");
        main_fn_scope.raw("\tprintln!(\"Started http server: 127.0.0.1:8088\");");
        main_fn_scope.raw("\tlet _ = sys.run();");
        main_fn_scope.raw("}");

        main_fn_scope.to_string().replace("\n\n", "\n")
}


#[cfg(test)]
mod tests {
    use workers::code_gen::CodeGen;
    use models::{ColumnDef, DataTypes};
    use codegen::{Block, Formatter, Function, Impl, Scope, Struct};

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
        let db_actor = "use actix::prelude::*;\nuse sqlite::Connection;\n\npub struct DbExecutor(pub Connection);\n\nimpl Actor for DbExecutor {\n    type Context = SyncContext<Self>;\n}".to_string();
        assert_eq!(db_actor, CodeGen::generate_db_actor());
    }

    #[test]
    fn generate_webservice_main() {
        let main_src = "pub mod actors;\npub mod models;\n\nextern crate clap;\nextern crate dotenv;\nextern crate env_logger;\nextern crate actix;\nextern crate actix_web;\nextern crate sqlite;\nextern crate futures;\n#[macro_use]\nextern crate serde_derive;\n\nuse actix::{Addr,Syn};\nuse actix::prelude::*;\nuse actors::db_actor::*;\nuse actix_web::{http, App, AsyncResponder, HttpRequest, HttpResponse, Error, Json};\nuse actix_web::server::HttpServer;\nuse futures::Future;\nuse actix_web::middleware::Logger;\n\n/// This is state where we will store *DbExecutor* address.\nstruct State {\n    db: Addr<Syn, DbExecutor>,\n}\n\n/// Used to implement all of the route handlers\nstruct RouteHandlers;\n\nimpl RouteHandlers {\n    fn index(_req: HttpRequest<State>) -> &\'static str {\n        \"Put the next steps instructions here\"\n    }\n}\n\nfn main() {\n\tstd::env::set_var(\"RUST_LOG\", \"actix_web=info\");\n\tenv_logger::init();\n\tlet sys = actix::System::new(\"csv2api\");\n// Start 3 parallel db executors\n\tlet addr = SyncArbiter::start(3, || {\n\t    DbExecutor(sqlite::open(\"test.db\").unwrap())\n\t});\n\tHttpServer::new(move || {\n\t\tApp::with_state(State{db: addr.clone()})\n\t\t\t.middleware(Logger::default())\n\t\t\t.resource(\"/\", |r| r.method(http::Method::GET).f(RouteHandlers::index))\n\t})\n\t.bind(\"127.0.0.1:8088\").unwrap()\n\t.start();\n\n\tprintln!(\"Started http server: 127.0.0.1:8088\");\n\tlet _ = sys.run();\n}".to_string();
        let res = CodeGen::generate_webservice("test.db".to_string(),&vec![]);
        assert_eq!(main_src, res);
    }

    #[test]
    fn generate_handler() {
        let mut expected_scope = Scope::new();
        let mut expected_impl = Impl::new("test");
        let mut expected_fn = Function::new("MyEntity");
        expected_fn
            .arg("_req", "HttpRequest<State>")
            .ret("&'static str")
            .line(format!("\"Pretend this is a list of MyEntity\""));
        expected_impl.push_fn(expected_fn);
        expected_scope.push_impl(expected_impl);
        let expected = expected_scope.to_string();


        let mut actual_scope = Scope::new();
        let mut actual_impl = Impl::new("test");
        let actual_fn = CodeGen::generate_handler("MyEntity");
        actual_impl.push_fn(actual_fn);
        actual_scope.push_impl(actual_impl);
        let actual = actual_scope.to_string();

        assert_eq!(actual, expected);
    }
}