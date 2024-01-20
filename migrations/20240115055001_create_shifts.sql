-- Add migration script here
CREATE TABLE IF NOT EXISTS shifts (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id),
    location_id UUID REFERENCES locations(id),
    start_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    end_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    timezone VARCHAR(255),
    rrule TEXT,
    notes TEXT,
    extended_props JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE
);
