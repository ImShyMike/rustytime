-- Revert admin_level smallint back to is_admin boolean
ALTER TABLE users
    ALTER COLUMN admin_level TYPE BOOLEAN
        USING CASE
            WHEN admin_level IS NULL THEN NULL
            WHEN admin_level <> 0 THEN TRUE
            ELSE FALSE
        END,
    RENAME COLUMN admin_level TO is_admin;
