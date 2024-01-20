-- Add migration script here
CREATE TABLE IF NOT EXISTS organization_invites (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id),
    invitee_email VARCHAR(255) NOT NULL,
    expiration_timestamp TIMESTAMP WITH TIME ZONE,
    accepted_timestamp TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
