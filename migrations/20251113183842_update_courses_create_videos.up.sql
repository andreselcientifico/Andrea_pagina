-- Add up migration script here
-- Renombrar columna 'name' a 'title'
ALTER TABLE courses RENAME COLUMN name TO title;

-- Agregar nuevas columnas
ALTER TABLE courses
ADD COLUMN long_description TEXT,
ADD COLUMN level TEXT CHECK (level IN ('básico', 'intermedio', 'avanzado')) DEFAULT 'básico',
ADD COLUMN duration TEXT,
ADD COLUMN students INT DEFAULT 0,
ADD COLUMN rating NUMERIC(2,1) DEFAULT 5,
ADD COLUMN image TEXT,
ADD COLUMN category TEXT CHECK (category IN ('básico', 'premium')) DEFAULT 'básico',
ADD COLUMN features JSONB DEFAULT '[]'::jsonb;

CREATE TABLE videos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID REFERENCES courses(id) ON DELETE CASCADE,
    "order" INT NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    duration TEXT,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

-- Índice para consultas por curso
CREATE INDEX idx_videos_course_id ON videos(course_id);
