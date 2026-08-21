-- Add migration script here
CREATE TABLE users(
    user_id uuid PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    status TEXT DEFAULT 'inactive' NOT NULL,
    password_hash TEXT NOT NULL,
    created_at timestamptz NOT NULL
);