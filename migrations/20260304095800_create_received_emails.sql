-- Tabla de correos recibidos (Bandeja de Entrada Admin)
CREATE TABLE IF NOT EXISTS received_emails (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resend_email_id VARCHAR(255) UNIQUE NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    text_content TEXT,
    html_content TEXT,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_received_emails_created_at ON received_emails(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_received_emails_is_read ON received_emails(is_read);
