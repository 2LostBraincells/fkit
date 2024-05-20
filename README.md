# Fetch Kit - FKIT

FKIT is a local solution for easily storing data gathered by your application.

## Installation

### Dependencies

#### Necessary Dependencies

* rustc and cargo - FKIT is written in Rust, so you will need the Rust compiler and package manager to build it.

#### Optional Dependencies

* sqlite3 - In its current state, FKIT uses sqlite3 as its database. This might change later. In the programs current state, the sqlite3 CLI is required to be able to view and retrieve the data in the database.

After installing all necessary Dependencies, you can try FKIT by running:

```bash
$ git clone https://github.com/2lostbraincells/fkit.git
```

## Usage

FKIT relies on a simple API to communicate between you application and the database. It uses the HTTP protocol to recieve data and respond. In its current state, FKIT can not retrieve the data in the database. If you need to retrieve the data, please use the sqlite3 CLI.

### Running

FKIT need a config file to run. The program will look for a file named `fkit.toml` in the current directory. If it does not find one, it will not run. You can create a default config file using the following command:

> If you are using cargo run, the sub command needs to be after the `--` flag which specifies to cargo that the following arguments are passed into the binary.

```bash
$ fkit init
```

After you've created the config file, you can run the program using the following command:

```bash
$ fkit run
```

### Config

The config file can be used to specify the location of the database file, as well as the port that the program will run on. You can run the command:

```bash
$ fkit --config-help
```
To get basic information on how to configure the program.

### API

For these examples, we will use the `curl` command to send HTTP requests. We will also use the default port 3000, but this can be changed in the config file.
To create a new project, you can send a post to the following endpoint:
    
```bash
$ curl -X POST http://localhost:3000/new/project_name
```

Where `project_name` is the name of the project you want to create.
This will create a new project in the database and return the string "success" if it was successful. This step is technically unnecessary but can be used if you want to create projects explicitly. 

To add data to the database, you can send a post to the following endpoint:

```bash
$ curl -X POST http://localhost:3000/add/project_name?column_name=value
```

Where `project_name` is the name of the project you want to add data to, `column_name` is the name of the column you want to add data to, and `value` is the value you want to add to the column. This will add the data to the database. In its current version, all data is stored as raw text. If the project or column does not exist, it will be added to the database automatically. 

## Future

> These are just some future plans if anyone is interested. Although these things will only happen if this project isn't fully abandoned :eyes: 

In the future FKIT will use a different database system as sqlite3 is not a great choice for database. It's quite limited in functionality and possible queries. Its simplicity and the fact that it runs local made it a good choice for this prototype.

Of course, there will be a way of retrieving the data from the database through the API. This will most likely be in the form of a CSV.

In the far future, FKIT could have a web interface to view and manage data as well as seeing live graphs of the data.











