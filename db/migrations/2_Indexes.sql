-- Delete duplicate user_entry_settings. `rowid` is probably an sqlite-ism?
DELETE FROM user_entry_settings
	WHERE rowid NOT IN (
		SELECT MIN(rowid)
		FROM user_entry_settings
		GROUP BY user_id, entry_id
	);

CREATE UNIQUE INDEX key ON user_entry_settings (user_id, entry_id);
