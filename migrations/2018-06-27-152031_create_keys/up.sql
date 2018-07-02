-- Your SQL goes here
CREATE TABLE keys(
        fingerprint bytea PRIMARY KEY,
        pgpkey bytea NOT NULL
);