-- Add migration script here
CREATE TABLE IF NOT EXISTS org_locations (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id),
    location_id UUID REFERENCES locations(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE
);
