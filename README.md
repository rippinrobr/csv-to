_Currently under development, not fit to be used by anyone else, and it's barely fit to be used by me_
Really, I mean it. I'm in the 'follow the white rabbit phase'.

# csv2api
Goal is to convert CSV file(s) into a database backed REST API

## The plan

### Phase 1 - Research/POC 
I want to write the code, with hard-coded parameters, big main function, etc. The goal is to get it
working end to end.  Once that is done I will start working on refactoring.

* ~~Generate Rust Structs that match the parsed CSV file(s)~~
* ~~Generate SQLite create table sql for each file~~
* ~~Load data into newly created tables returning number of records inserted~~
* Add ~~TOML~~ YAML so users can set the parameters for all the workers read from csv2api.toml by default
* Add command line option to use a different TOML file
* Generate code for Basic HTTP server that responds to a the route /
* Generate base routes for all newly created structs only return a string
* Generate Select * code for all tables

### Phase 2 - Refactoring towards Actors
Goals: Add more tests,  Clean up the main function so its not a mess, convert workers to actors, and create handlers for the base URLs using actix-web

* clean up main, breaking out functions as needed
* convert parse_csv.rs to an actor
* convert code_gen.rs to an actor 
* convert sql_gen.rs to an actor
* create a db.rs actor that uses the sqlite.rs worker
* create handlers for routes that actually retreive data from the database

### Phase 3 - Data Cleansing/Rules 
Goals: Add the ability to clean data with rules, renaming/creation of columns, and schema related modifications
* Add ability to add unique constraints to a table based on the file parsed and the columns - map filename to unique with columns included. Something like <filename>: [ unique col1, col2, col3 ]
* Add ability to add a column to a table based on the file name - map file name to colum desc
* Add ability to associate code/rules with a particular table and/or column(s)
* Add ability to map an incoming column to a new column in the database

### Phase 4 - Add Postgres Support
* Update the db layer generation to handle multiple db types
* Create a postgres worker
* update db actor to support postgres
