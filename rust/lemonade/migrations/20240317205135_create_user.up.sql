-- Add up migration script here
create table if not exists lemonade.user
(
    id   uuid not null
        constraint user_pk
            primary key,
    name text not null
);


