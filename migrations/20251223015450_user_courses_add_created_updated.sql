-- Add migration script here
ALTER TABLE user_courses
ALTER COLUMN purchase_at TYPE TIMESTAMPTZ USING purchase_date AT TIME ZONE 'UTC';

ALTER TABLE user_courses
ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

ALTER TABLE user_courses
ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
