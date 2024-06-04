-- Additional user fields
-- This remains dumb because sqlite is dumb

CREATE TABLE users_temp (
	id TEXT NOT NULL PRIMARY KEY,
	name TEXT NOT NULL,
	password_hash TEXT NOT NULL,
	user_type TEXT NOT NULL
);

INSERT INTO users_temp (
	id, name, password_hash, user_type
) SELECT id, name, password_hash, 'Member'
	FROM users;

DROP TABLE users;

ALTER TABLE users_temp RENAME TO users;

-- A note about 1_User_entry_settings.sql: that script is buggy, and the order
-- of operations here (create new, copy rows, drop original, rename new) is
-- correct. Otherwise, sqlite will corrupt references to the original table when
-- you rename it. (Why can you drop a table with active foreign keys against it?
-- Truly a question for the ages.)

