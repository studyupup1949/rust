-- Create Activation Tokens Table
CREATE TABLE activation_tokens(
    activation_token TEXT NOT NULL,
    user_id uuid NOT NULL REFERENCES users (user_id),
    created_at timestamptz NOT NULL,
    PRIMARY KEY (activation_token)
);
