INSERT INTO location_invites (id, location_id, invitee_email, expiration_timestamp, accepted_timestamp, created_at)
VALUES
('c81e728d-9d4c-3f63-af06-7f89cc14862c', '45c48cce-2e2d-41d7-9963-00e04c68d011', 'invitee1@example.com', NOW() + INTERVAL '7 DAYS', NULL, NOW());
