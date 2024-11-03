-- Add down migration script here
DROP TRIGGER trigger_nordigen_tokens_changed ON nordigen_tokens;

DROP FUNCTION notify_nordigen_tokens;

DROP TABLE nordigen_tokens;
