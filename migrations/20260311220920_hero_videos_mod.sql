-- Add migration script here
ALTER TABLE hero_videos 
RENAME COLUMN video_id TO video_url;

ALTER TABLE hero_videos
ADD COLUMN date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN eventtype VARCHAR(50) NOT NULL DEFAULT 'otro' CHECK (eventtype IN ('bodas', 'cumpleaños', 'festival', 'corporativo', 'serenata' , 'otro')),
ADD COLUMN thumbnail_url VARCHAR(255) NOT NULL DEFAULT '';