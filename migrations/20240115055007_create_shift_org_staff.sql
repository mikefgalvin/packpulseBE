-- Add migration script here
CREATE TABLE IF NOT EXISTS shift_org_staff (
    id UUID PRIMARY KEY,
    shift_id UUID REFERENCES shifts(id),
    org_staff_id UUID REFERENCES org_staff(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE
);
