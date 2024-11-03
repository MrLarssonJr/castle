-- Add up migration script here
create table users
(
    id            uuid not null
        primary key,
    username      text not null
        constraint users_pk
            unique,
    password_hash text not null
);
