INSERT INTO shifts (id, organization_id, location_id, start_timestamp, end_timestamp, timezone, rrule, notes, extended_props, created_at, updated_at)
VALUES
('a87ff679-a2f3-371d-9181-a67b7542122c', '1679091c-5a88-442f-9383-0a9f2a02a8a9', '45c48cce-2e2d-41d7-9963-00e04c68d011', NOW(), NOW() + INTERVAL '1 HOUR', 'UTC', 'RRULE FREQ=WEEKLY; BYDAY=MO', 'Notes 1', '{}', NOW(), NOW()),
('e4da3b7f-bbce-3d42-9f6e-7d4f22d5c3df', '8f14e45f-e8f1-4d26-bfcf-44ce29ed702a', 'c20ad4d7-6fe8-446f-aed8-6e5ac6c7ae4d', NOW(), NOW() + INTERVAL '1 HOUR', 'UTC', 'RRULE FREQ=WEEKLY; BYDAY=TU', 'Notes 2', '{}', NOW(), NOW());
