-- Add migration script here
-- Note : This is a transaction to ensure atomicity
BEGIN;
    UPDATE subscriptions    
        SET status = 'confirmed'
        where status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
    