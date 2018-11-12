use codegen::{Block, Function, Impl, Scope, Struct};
use models::ColumnDef;
use std::io::Error;

pub enum CodeGenTarget {
    CurlScript,
    DbActor,
    Handler,
    ModFile,
    Struct,
    WebService,
}

pub struct CodeGen;


impl CodeGen {
    
    pub fn generate_handler(name: &str) -> Function {
        let mut myfn = Function::new(&name.to_lowercase());
        myfn
            .arg("req", "HttpRequest<State>")
            .ret("impl Future<Item=HttpResponse, Error=Error>")
            .line(&format!("use actors::{}::*;\n", name.to_lowercase()))
            .line(format!("\treq.state().db.send({}Msg{{page_num: 1}})", name))
            .line("\t\t.from_err()");
        let mut and_then_block = Block::new(".and_then(|res| ");
        let mut match_block = Block::new("\tmatch res ");
        match_block.line("Ok(i) => Ok(HttpResponse::Ok().json(i)),");
        
        let mut error_block = Block::new("Err(e) => ");
        error_block.line(&format!("eprintln!(\"get_{} error: {{}}]\",e);", name.to_lowercase()));
        error_block.line("Ok(HttpResponse::InternalServerError().into())");
        match_block.push_block(error_block);
        and_then_block.push_block(match_block);
        and_then_block.after(").responder()");

        myfn.push_block(and_then_block);
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
            my_model.field(&format!("pub {}", &c.name.to_lowercase()), c.data_type.string());
        }
        
        scope.push_struct(my_model);
        
        scope.to_string()
    }

    pub fn create_handler_actor(struct_name: &str) -> String { //struct_meta: &(String, Vec<ColumnDef>)) -> String {
        let mut scope = Scope::new();
        //let struct_name = &struct_meta.0;
        
        for (u0, u1) in vec![("actix::prelude", "*"), ("db", "DB"), (&format!("models::{}", struct_name.to_lowercase()), "*"), ("super::db", "DbExecutor")] {
            scope.import(u0, u1);
        }

        // Creating the message struct
        let msg_struct_name = &format!("{}Msg", struct_name);
        let mut msg_struct = Struct::new(msg_struct_name);
        msg_struct.doc(&format!("Message for returning a paged list of {} records", struct_name));
        msg_struct.field("pub page_num", "u32");
        msg_struct.vis("pub");
        scope.push_struct(msg_struct);

        // impl for Message on the struct 
        let mut msg_impl = Impl::new(&format!("{}Msg", struct_name));
        msg_impl.impl_trait("Message");
        msg_impl.associate_type("Result", &format!("Result<Vec<{}>, String>", struct_name));
        scope.push_impl(msg_impl);
        
        // This is for the Handler for the DbExecutor
        // impl Handler<Conspiracies> for DbExecutor {
        let mut handler_impl = Impl::new("DbExecutor");
        handler_impl.impl_trait(&format!("Handler<{}Msg>", struct_name));
        handler_impl.associate_type("Result", &format!("Result<Vec<{}>, String>", struct_name));

        let mut impl_func = Function::new("handle");
        impl_func.arg_mut_self();
        impl_func.arg("msg", msg_struct_name);
        impl_func.arg("_", "&mut Self::Context");
        impl_func.ret("Self::Result");
        impl_func.line(&format!("\tDB::get_{}(&self.0, msg.page_num)", struct_name.to_lowercase()));
        
        handler_impl.push_fn(impl_func);
        scope.push_impl(handler_impl);
        
        scope.to_string()
    }

    pub fn generate_mod_file_contents(mod_names: &Vec<String>) -> String{
        let mut scope = Scope::new();

        for file_name in mod_names.iter() {
            scope.raw(&format!("pub mod {};", file_name.to_lowercase().replace(".rs", "")));
        }

        scope.to_string()
    }

    pub fn generate_mod_file(entities: &Vec<String>) -> String {
        let mut scope = Scope::new();
        
        for entity in entities {
            scope.raw(&format!("pub mod {};", entity.to_lowercase()));
        }
        
        scope.to_string().replace("\n\n", "\n") + "\n"
    }

    pub fn generate_db_actor() -> String {
        let mut scope = Scope::new();

        scope.import("actix::prelude", "*");
        scope.import("rusqlite", "Connection");
        scope.raw("pub struct DbExecutor(pub Connection);");
        
        let mut actor_trait = Impl::new("DbExecutor");
        actor_trait.impl_trait("Actor");
        actor_trait.associate_type("Context", "SyncContext<Self>");

        scope.push_impl(actor_trait);
    
        scope.to_string()
    }

    pub fn generate_webservice(db_path: String, entities: &Vec<String>) -> String {
        let mut scope = Scope::new();
        
        for use_stmt in vec![("actix", "{Addr,Syn}"), ("actix::prelude", "*"), ("actors::db", "*"), ("actix_web", "http, App, AsyncResponder, HttpRequest, HttpResponse"),
                            ("actix_web::server", "HttpServer"), ("futures", "Future"), ("actix_web", "Error"), ("actix_web", "Json"), ("actix_web::middleware", "Logger"),
                            ("rusqlite", "Connection"), ("models", "*")] {
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
            handler_impl.push_fn(CodeGen::generate_handler(&ent));
        }
        scope.push_impl(handler_impl);
        scope.raw("");

        create_extern_create_defs() + &scope.to_string() + &create_main_fn(db_path, &entities)
    }

    pub fn create_curl_script(output_dir: &str, entities: &Vec<String>) -> Result<String, Error> {
        let mut scope = Scope::new();
        scope.raw("#!/bin/bash\n");
        for ent in entities {
                let lower_ent = ent.to_lowercase().replace(".rs", "");
                scope.raw(&format!("curl http://localhost:8088/{}", lower_ent));
        }

        return super::write_code_to_file(output_dir, "curl_test.sh", scope.to_string().replace("\n\n", "\n"))    
    }

}

fn create_extern_create_defs() -> String {
    let mut extern_scope = Scope::new(); 
        for extern_crate in vec!["pub mod actors;\npub mod db;\npub mod models;\n\n\nextern crate clap;", "extern crate dotenv;", "extern crate env_logger;", "extern crate actix;", 
                                "extern crate actix_web;", "extern crate rusqlite;", "extern crate futures;", "#[macro_use]", "extern crate serde_derive;"] {
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
        main_fn_scope.raw(&format!("\t    DbExecutor(Connection::open(\"{}\").unwrap())", db_path));
        main_fn_scope.raw("\t});");

        main_fn_scope.raw("\tHttpServer::new(move || {");
        main_fn_scope.raw("\t\tApp::with_state(State{db: addr.clone()})");
        main_fn_scope.raw("\t\t\t.middleware(Logger::default())");
        main_fn_scope.raw("\t\t\t.resource(\"/\", |r| r.method(http::Method::GET).f(RouteHandlers::index))");

        for ent in entities {
            let lower_ent = ent.to_lowercase();
            main_fn_scope.raw(&format!("\t\t\t.resource(\"/{}\", |r| r.method(http::Method::GET).a(RouteHandlers::{}))", lower_ent, lower_ent));
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
    use super::*;
    use code_gen::CodeGen;
    use models::{ColumnDef, DataTypes};
    use codegen::{Impl, Scope};

    #[test]
    fn generate_mod_file() {
        let expected = "pub mod sqlite;\npub mod code_gen;\npub mod sql_gen;\npub mod sqlite_code_gen;\npub mod config;\npub mod output;\npub mod input;\npub mod parse_csv;\n".to_string();
        let actual = CodeGen::generate_mod_file(&vec!["./src/workers".to_string()]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn create_handler_actor() {
        let expected_len = 520;
        let actual = CodeGen::create_handler_actor("my_actor");

        assert_eq!(actual.len(), expected_len);
    }

    #[test] 
    fn generate_struct() {
        let struct_def = "#[derive(Debug, Deserialize, Serialize)]\npub struct people {\n    pub name: String,\n    pub age: i64,\n    pub weight: f64,\n}".to_string();
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
        let db_actor = "use actix::prelude::*;\nuse rusqlite::Connection;\n\npub struct DbExecutor(pub Connection);\n\nimpl Actor for DbExecutor {\n    type Context = SyncContext<Self>;\n}".to_string();
        assert_eq!(db_actor, CodeGen::generate_db_actor());
    }

    #[test]
    fn generate_webservice_main() {
        let expected_len = 1398;
        let actual = CodeGen::generate_webservice("test.db".to_string(),&vec![]);

        assert_eq!(actual.len(), expected_len);
    }

    #[test]
    fn generate_handler() {
        let expected_len = 541;

        let mut actual_scope = Scope::new();
        let mut actual_impl = Impl::new("test");
        let actual_fn = CodeGen::generate_handler("MyEntity");
        actual_impl.push_fn(actual_fn);
        actual_scope.push_impl(actual_impl);
        let actual = actual_scope.to_string();

        assert_eq!(actual.len(), expected_len);
    }
}