-- Initialize the Russet database

CREATE TABLE feeds (
	id TEXT NOT NULL PRIMARY KEY,
	url TEXT NOT NULL,
	title TEXT NOT NULL
);

CREATE TABLE entries (
	id TEXT NOT NULL PRIMARY KEY,
	feed_id TEXT NOT NULL,
	internal_id TEXT NOT NULL,
	fetch_index INT NOT NULL,
	article_date INT NOT NULL,
	title TEXT NOT NULL,
	url TEXT NULL,
	FOREIGN KEY (feed_id) REFERENCES feeds(id)
) STRICT;

CREATE TABLE users (
	id TEXT NOT NULL PRIMARY KEY,
	name TEXT NOT NULL,
	password_hash TEXT NOT NULL
);

CREATE TABLE sessions (
	token TEXT NOT NULL PRIMARY KEY,
	user_id TEXT NOT NULL,
	expiration INT NOT NULL,
	FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE subscriptions (
	user_id TEXT NOT NULL,
	feed_id TEXT NOT NULL,
	FOREIGN KEY (user_id) REFERENCES users(id),
	FOREIGN KEY (feed_id) REFERENCES feeds(id)
);

CREATE TABLE user_entry_settings (
	user_id TEXT NOT NULL,
	entry_id TEXT NOT NULL,
	read TIMESTAMP WITH TIME ZONE NULL,
	tombstone BOOLEAN NOT NULL DEFAULT FALSE,
	FOREIGN KEY (user_id) REFERENCES users(id),
	FOREIGN KEY (entry_id) REFERENCES entries(id)
);

CREATE TABLE metadata (
	fetch_index INT NOT NULL
);
INSERT INTO metadata (fetch_index) VALUES (0);
