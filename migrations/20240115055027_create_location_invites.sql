-- Add migration script here
CREATE TABLE IF NOT EXISTS location_invites (
    id UUID PRIMARY KEY,
    location_id UUID REFERENCES locations(id),
    invitee_email VARCHAR(255) NOT NULL,
    expiration_timestamp TIMESTAMP WITH TIME ZONE,
    accepted_timestamp TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
