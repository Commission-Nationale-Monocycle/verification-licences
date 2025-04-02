CREATE TABLE membership
(
    id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    last_name           VARCHAR NOT NULL,
    first_name          VARCHAR NOT NULL,
    gender              VARCHAR NOT NULL,
    birthdate           VARCHAR,
    age                 INTEGER,
    membership_number   VARCHAR NOT NULL,
    email_address       VARCHAR NOT NULL,
    payed               BOOLEAN NOT NULL,
    end_date            VARCHAR NOT NULL,
    expired             BOOLEAN NOT NULL,
    club                VARCHAR NOT NULL NOT NULL,
    structure_code      VARCHAR NOT NULL NOT NULL
)