-- Create Reset Password Tokens Table
CREATE TABLE password_reset_tokens(
    reset_token TEXT NOT NULL,
    user_id uuid NOT NULL REFERENCES users (user_id),
    created_at timestamptz NOT NULL,
    PRIMARY KEY (reset_token)
);