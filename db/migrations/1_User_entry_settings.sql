-- Fix a bunch of dumb sqlite-isms that weren't being exercised
-- This is dumb like this because sqlite is dumb.
ALTER TABLE user_entry_settings RENAME TO user_entry_settings_temp;

CREATE TABLE user_entry_settings (
	user_id TEXT NOT NULL,
	entry_id TEXT NOT NULL,
	read INT NULL,
	tombstone INT NULL,
	FOREIGN KEY (user_id) REFERENCES users(id),
	FOREIGN KEY (entry_id) REFERENCES entries(id)
);

INSERT INTO user_entry_settings (
	user_id, entry_id, read, tombstone
) SELECT user_id, entry_id, read, tombstone
	FROM user_entry_settings_temp;

DROP TABLE user_entry_settings_temp;

