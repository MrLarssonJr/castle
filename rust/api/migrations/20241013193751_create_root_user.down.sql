-- Add down migration script here
DELETE
FROM roles
WHERE user_id = '01928766-3a26-7390-b099-a1a595514847';

DELETE
FROM users
WHERE id = '01928766-3a26-7390-b099-a1a595514847';

