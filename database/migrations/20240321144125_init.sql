-- Add migration script here

CREATE TABLE Dataset (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW
);

CREATE TABLE Datastream (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    dataset_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW,

    FOREIGN KEY (dataset_id) REFERENCES Dataset(id)
);

CREATE TABLE Collection (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    dataset_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW,

    FOREIGN KEY (dataset_id) REFERENCES Dataset(id)
);

CREATE TABLE Raw_Data (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    data BLOB NOT NULL,
    datastream_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL,

    FOREIGN KEY (datastream_id) REFERENCES Datastream(id),
    FOREIGN KEY (collection_id) REFERENCES Collection(id)
);

