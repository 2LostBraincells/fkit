# Fetch Kit - FKIT

FKIT is a local solution for easily storing data gathered by your application.

## Installation

### Dependencies

* sqlite3 - In its current state, FKIT uses sqlite3 as its database. This might change later.
* rustc and cargo - FKIT is written in Rust, so you will need the Rust compiler and package manager to build it.

After installing all necessary Dependencies, you can try FKIT by running:

```bash
$ git clone https://github.com/2lostbraincells/fkit.git
```

## Usage

Fkit relies on a simple API to communicate between you application and the database. It uses the HTTP protocol to recieve data and respond. In its current state, FKIT can not retrieve the data in the database. If you need to retrieve the data, please use the sqlite3 CLI.

### API

For these examples, we will use the `curl` command to send HTTP requests. We will also use the default port 3000, but this can be changed in the config file.
To create a new project, you can send a post to the following endpoint:
    
```bash
$ curl -X POST http://localhost:3000/new/project_name
```

Where `project_name` is the name of the project you want to create.
This will create a new project in the database and return the string "success" if it was successful. This step is technically unnecessary.

To add data to the database, you can send a post to the following endpoint:

```bash
$ curl -X POST http://localhost:3000/add/project_name?column_name=value
```

Where `project_name` is the name of the project you want to add data to, `column_name` is the name of the column you want to add data to, and `value` is the value you want to add to the column. This will add the data to the database. In its current version, all data is stored as raw text.

