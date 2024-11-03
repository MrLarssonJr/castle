-- Add up migration script here
create table sessions
(
    id         uuid not null
        constraint sessions_pk
            primary key,
    user_id    uuid not null
        constraint sessions_users_id_fk
            references users,
    token_hash text not null
);


