-- Add migration script here
ALTER TABLE user_achievement
ADD CONSTRAINT user_achievement_unique
UNIQUE (user_id, achievement_id);
