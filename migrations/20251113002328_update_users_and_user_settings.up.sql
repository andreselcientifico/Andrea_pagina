-- Add up migration script here
ALTER TABLE users
ADD COLUMN phone VARCHAR(32),
ADD COLUMN location VARCHAR(255),
ADD COLUMN bio TEXT,
ADD COLUMN birth_date DATE,
ADD COLUMN profile_image_url TEXT;