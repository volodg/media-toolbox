CREATE TABLE users
(
    id BIGSERIAL PRIMARY KEY,
    name text NOT NULL,
    about text NOT NULL,
    email text NOT NULL,
    CONSTRAINT email UNIQUE (email)
);