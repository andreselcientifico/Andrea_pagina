-- Add migration script here
ALTER TABLE user_lesson_progress
ADD CONSTRAINT unique_user_lesson UNIQUE (user_id, lesson_id);
ALTER TABLE course_progress
ADD CONSTRAINT unique_user_course UNIQUE (user_id, course_id);
