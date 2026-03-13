-- Tabla de solicitudes de eventos
CREATE TABLE IF NOT EXISTS event_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(32),
    event_type VARCHAR(50) NOT NULL, -- boda, cumpleaños, festival, corporativo, serenata, otro
    event_date DATE,
    location VARCHAR(255),
    guests INT,
    message TEXT,
    budget VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, contacted, confirmed, rejected
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_requests_status ON event_requests(status);
CREATE INDEX IF NOT EXISTS idx_event_requests_created_at ON event_requests(created_at DESC);
