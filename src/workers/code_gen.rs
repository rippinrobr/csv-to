use actix::prelude::*;
use csv_converter::code_gen::{CodeGen};
use csv_converter::models::ParsedContent;


pub struct Generator;

// Messages and Actors from here down
pub struct CodeGenStruct {
    pub struct_name: String,
    pub models_dir: String, 
    pub parsed_content: ParsedContent,
}

impl Message for CodeGenStruct {
    type Result = String;
}

impl Handler<CodeGenStruct> for Generator {
    type Result = String;

    fn handle(&mut self, msg: CodeGenStruct, _: &mut Context<Self>) -> Self::Result {
        let file_name = format!("{}.rs", &msg.struct_name.to_lowercase());
        let struct_src = CodeGen::generate_struct(&msg.struct_name, &msg.parsed_content.columns);
        
        match CodeGen::write_code_to_file(&msg.models_dir, &file_name, struct_src.clone()) {
            Err(e) => eprintln!("ERROR: {} trying to write {}/{}", e, &msg.models_dir, file_name),
            Ok(_) => print!("Created {}/{}", &msg.models_dir, file_name)
        };
        
        "".to_string()
    }
}

pub struct CodeGenHandler {
    pub struct_name: String,
    pub actors_dir: String, 
    pub parsed_content: ParsedContent,
}

impl<'a> Message for CodeGenHandler {
    type Result = String;
}

impl Handler<CodeGenHandler> for Generator {
    type Result = String;

    fn handle(&mut self, msg: CodeGenHandler, _: &mut Context<Self>) -> Self::Result {
        let file_name = format!("{}.rs", &msg.struct_name.to_lowercase());
        let actor_code = CodeGen::create_handler_actor(&msg.struct_name);

        match CodeGen::write_code_to_file(&msg.actors_dir, &file_name, actor_code){
            Err(e) => eprintln!("ERROR: {} trying to write {}/{}", e, &msg.actors_dir, file_name),
            Ok(_) => print!("Created {}/{}", &msg.actors_dir, file_name)
        };

        "".to_string()
    }
}

pub struct CodeGenDbActor{
    pub db_src_dir: String,
    pub file_name: String,
}

impl Message for CodeGenDbActor {
    type Result = String;
}

impl Handler<CodeGenDbActor> for Generator {
    type Result = String;
    
    fn handle(&mut self, msg: CodeGenDbActor, _: &mut Context<Self>) -> Self::Result {
        let actor_code = CodeGen::generate_db_actor();

        match CodeGen::write_code_to_file(&msg.db_src_dir, &msg.file_name, actor_code){
            Err(e) => eprintln!("ERROR: {} trying to write {}/{}", e, &msg.db_src_dir, &msg.file_name),
            Ok(_) => print!("Created {}/{}", &msg.db_src_dir, &msg.file_name)
        };

        "".to_string()
    }
}

impl Actor for Generator {
    type Context = Context<Self>;
}
