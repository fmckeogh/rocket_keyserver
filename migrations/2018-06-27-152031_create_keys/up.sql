-- Your SQL goes here
CREATE TABLE keys(
        fingerprint CHAR(40) PRIMARY KEY,
        pgpkey TEXT NOT NULL
);