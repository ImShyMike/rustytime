-- Migrate is_admin boolean to admin_level smallint
ALTER TABLE users
    RENAME COLUMN is_admin TO admin_level;

ALTER TABLE users
    ALTER COLUMN admin_level DROP DEFAULT,
    ALTER COLUMN admin_level TYPE SMALLINT USING CASE WHEN admin_level THEN 1 ELSE 0 END,
    ALTER COLUMN admin_level SET DEFAULT 0,
    ALTER COLUMN admin_level SET NOT NULL;
