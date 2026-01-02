-- Add migration script here
-- Comentarios del curso
CREATE TABLE course_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    course_id UUID NOT NULL
        REFERENCES courses(id)
        ON DELETE CASCADE,

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    content TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índice para listar comentarios por curso
CREATE INDEX idx_course_comments_course_id
    ON course_comments(course_id);

-- Comentarios de lecciones
CREATE TABLE lesson_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    lesson_id UUID NOT NULL
        REFERENCES lessons(id)
        ON DELETE CASCADE,

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    content TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índice para listar comentarios por lección
CREATE INDEX idx_lesson_comments_lesson_id
    ON lesson_comments(lesson_id);

-- Calificaciones de cursos
CREATE TABLE course_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    course_id UUID NOT NULL
        REFERENCES courses(id)
        ON DELETE CASCADE,

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    rating INT NOT NULL CHECK (rating BETWEEN 1 AND 5),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Un usuario solo puede calificar una vez por curso
    CONSTRAINT unique_course_user_rating
        UNIQUE (course_id, user_id)
);

-- Índice para cálculos de promedio
CREATE INDEX idx_course_ratings_course_id
    ON course_ratings(course_id);

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
