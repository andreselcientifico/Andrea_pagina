-- Add migration script here
ALTER TABLE users
ADD COLUMN email_notifications BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN course_reminders   BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN new_content        BOOLEAN NOT NULL DEFAULT true;
