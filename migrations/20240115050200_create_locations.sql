CREATE TABLE IF NOT EXISTS locations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    access VARCHAR(50) CHECK (access IN ('private', 'public')) NOT NULL,
    type VARCHAR(50) CHECK (type IN ('community', 'event', 'commercial')) NOT NULL,
    address TEXT,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE
);
