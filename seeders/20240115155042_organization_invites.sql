INSERT INTO organization_invites (id, organization_id, invitee_email, expiration_timestamp, accepted_timestamp, created_at)
VALUES
('c4ca4238-a0b9-3382-8dcc-509a6f75849b', '1679091c-5a88-442f-9383-0a9f2a02a8a9', 'invitee1@example.com', NOW() + INTERVAL '7 DAYS', NULL, NOW());