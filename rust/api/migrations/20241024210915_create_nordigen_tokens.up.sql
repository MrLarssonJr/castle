-- Add up migration script here
create table nordigen_tokens
(
    secret_id          text not null
        constraint nordigen_tokens_pk
            primary key,
    access             text,
    access_expires_at  timestamp with time zone,
    refresh            text,
    refresh_expires_at timestamp with time zone,
    constraint all_or_none_null
        check (((access IS NULL) AND (access_expires_at IS NULL) AND (refresh IS NULL) AND
                (refresh_expires_at IS NULL)) OR
               ((access IS NOT NULL) AND (access_expires_at IS NOT NULL) AND (refresh IS NOT NULL) AND
                (refresh_expires_at IS NOT NULL)))
);

CREATE OR REPLACE FUNCTION notify_nordigen_tokens() RETURNS TRIGGER AS
$$
BEGIN
    NOTIFY nordigen_tokens;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

create trigger trigger_nordigen_tokens_changed
    after insert or update or delete
    on nordigen_tokens
    for each row
execute procedure notify_nordigen_tokens();
