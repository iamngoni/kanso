-- Apple sign-in callback target: the scheme the app that started the flow registered.
-- Kanso Nightly and the development build have their own schemes, so the callback page
-- must return to the process that began the ceremony. Rows from before this migration
-- fall back to the release scheme.

ALTER TABLE auth_flows ADD COLUMN app_scheme TEXT;
