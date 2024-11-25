-- Add migration script here
CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    sunscriber_id uuid NOT NULL
        REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_token)
);