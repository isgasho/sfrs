CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    pw_cost INTEGER NOT NULL,
    pw_nonce VARCHAR NOT NULL,
    version VARCHAR NOT NULL
)