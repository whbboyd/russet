-- Unique key on subscriptions, so you can't accidentally subscribe to the same
-- feed twice. This also provides an index on subscriptions by user which can be
-- used by `get_userentries` (for e.g. the root page).
CREATE UNIQUE INDEX subscriptions_key ON subscriptions (user_id, feed_id);

-- For entries by feed; bake order into the index
CREATE INDEX entries_by_feed ON entries (feed_id, check_id DESC, article_date DESC);

