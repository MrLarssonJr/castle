CREATE TYPE role AS ENUM ('admin');

CREATE TABLE roles
(
    user_id uuid NOT NULL
        CONSTRAINT roles_pk
            PRIMARY KEY
        CONSTRAINT roles_users_id_fk
            REFERENCES users,
    role    ROLE NOT NULL
);

