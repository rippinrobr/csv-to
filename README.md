[![](https://circleci.com/gh/rippinrobr/csv-to/tree/csv-to-db.svg?style=svg)](https://circleci.com/gh/rippinrobr/csv-to)

_The new and improved csv-to will be released shortly.  There is one blocker that I need to resolve and then it will be 
merged to master and ready to use_

# csv-to
The goal of this project is to create a utility that reads a single file or a directory of csv files to create and load 
a database and eventually code around the database.

![Image of the results of csv-to db call][screen-shot]
## csv-to db  Create a db from csv file(s)
The `db` sub-command parses the files, creates a database table for each file it parses, and loads the database.  
Currently, `SQLite` and `Postgres` are supported.

```
creates and loads a database from CSV file(s)

USAGE:
    csv-to db [FLAGS] [OPTIONS] --connection-info <connection_info> --type <db_type> --name <name>

FLAGS:
        --drop-stores    Drops tables/collections if the already exist
    -h, --help           Prints help information
        --no-headers     The CSV file(s) have no column headers
    -V, --version        Prints version information

OPTIONS:
    -c, --connection-info <connection_info>    Database connectivity information
    -t, --type <db_type>                       The type of database to create, valid types are sqlite and postgres
    -d, --directories <directories>...         The directories that contain CSV files to be processed, a comma delimited
                                               string of paths
    -e, --extension <extension>                the file extension for the CSV files to be parsed [default: csv]
    -f, --files <files>...                     The CSV files to be processed, can be /path/to/files/ or a comma
                                               delimited string of paths
    -n, --name <name>                          Name of the database to be created
```

## csv-to data-layer - create a db and code to read from it
This sub-command will create a new db and code to read data from the database.  After running this command you'll have a
new database, code to access and run queries against the database along with models that, well model, the data.  Planned
languages for generated code are Rust, Go, and C#.

## csv-to api - generates a REST API for reading 
This sub-command will execute the two commands above and then generate a GET route for each table, returning paged 
response.  The API will be generated in Rust, Go, and C#.

_If there is a database or language you'd like to have support for open an issue with the your suggestion._
 

[screen-shot]: https://github.com/rippinrobr/csv-to/raw/master/assets/csv-to-db-results.png
