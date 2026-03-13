-- Create hero_videos table
CREATE TABLE hero_videos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    source VARCHAR(50) NOT NULL CHECK (source IN ('youtube', 'facebook')),
    video_id VARCHAR(255) NOT NULL,
    embed_url TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on source for faster querying
CREATE INDEX idx_hero_videos_source ON hero_videos(source);

-- Insert default video (YouTube)
INSERT INTO hero_videos (title, source, video_id, embed_url, description)
VALUES (
    'Presentación Academia',
    'youtube',
    'efFC9ROqTzM',
    'https://www.youtube-nocookie.com/embed/efFC9ROqTzM',
    'Conoce nuestra academia y metodología'
);
