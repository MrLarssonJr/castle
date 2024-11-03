-- Add up migration script here
INSERT INTO users (id, username, password_hash)
VALUES ('01928766-3a26-7390-b099-a1a595514847', 'root',
        '$argon2i$v=19$m=16,t=8,p=1$xFc8Ng6RdfD0ZJ+aR2OVaw$AnUHWHiiVKaPCBkP2yf+YplH+rfW0momVot+BdX9HY0');

INSERT INTO roles (user_id, role)
VALUES ('01928766-3a26-7390-b099-a1a595514847', 'admin');
