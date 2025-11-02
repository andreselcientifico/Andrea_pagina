-- Add down migration script here
DROP TABLE IF EXISTS course_progress;
DROP TABLE IF EXISTS user_course;
DROP TABLE IF EXISTS subscription;
DROP TABLE IF EXISTS notification;
DROP TABLE IF EXISTS user_achievement;
DROP TABLE IF EXISTS achievement;
DROP TABLE IF EXISTS user_settings;
DROP TABLE IF EXISTS users;

-- Drop enums
DROP TYPE IF EXISTS user_role;

-- Drop extensions
DROP EXTENSION IF EXISTS "uuid-ossp";
