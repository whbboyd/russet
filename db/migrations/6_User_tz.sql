-- Timezone field for user

ALTER TABLE users ADD COLUMN tz TEXT NOT NULL DEFAULT "UTC";
