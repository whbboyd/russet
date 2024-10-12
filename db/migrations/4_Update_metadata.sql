-- NOTE on the previous migration (`3_User_settings.sql`), placed here as not to
--	break migration checksums: because of a nasty interaction between sqlx's
--	desire to be safe and sqlite's desire to be dumb, it's not actually possible
--	to run that migration against a live install with data. sqlx sets
--	`PRAGMA foreign_keys = ON` (which is objectively the only sane behavior),
--	but sqlite provides no mechanism for actually manipulating those
--	constraints, so if there's data present in the table, you're in a bind.
--	sqlite advises setting `PRAGMA foreign-keys = OFF` to run the migration, but
--	that seems not to be possible in sqlx's migration path. The migration path I
--	ended up with was to run the migration against a temporary empty database,
--	copy the migration row into the live database, and apply the migration
--	script by hand.

-- Metadata for update scheduling

-- see ../../src/domain/feeds/update.rs

-- We need to track more information about checks than that there have been n of
-- them so far.
CREATE TABLE feed_checks (
	id INT NOT NULL PRIMARY KEY,
	feed_id TEXT NOT NULL,
	check_time INT NOT NULL,
	next_check_time INT NOT NULL,
	etag TEXT NULL,
	FOREIGN KEY (feed_id) REFERENCES feeds(id)
);
-- These entries are "fake", but we need something for the new foreign key in
-- feeds to reference
INSERT INTO feed_checks (
	id, feed_id, check_time, next_check_time, etag
) SELECT (metadata.fetch_index + feeds.rowid), id, 0, 0, NULL
	FROM feeds
	-- There's only one row in metadata
	JOIN metadata ON TRUE;
-- We're not storing any durable metadata anymore.
DROP TABLE metadata;

ALTER TABLE entries RENAME COLUMN fetch_index TO check_id;

