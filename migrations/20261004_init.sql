CREATE TYPE submission_status AS ENUM (
    'draft', 
    'queued', 
    'running',
    'complete'
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS submissions (
    id TEXT NOT NULL PRIMARY KEY, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status submission_status NOT NULL DEFAULT 'queued',
    created_by TEXT NOT NULL REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS github_associations (
    github_user_id BIGINT NOT NULL PRIMARY KEY,
    github_email TEXT NOT NULL,
    github_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_id TEXT NOT NULL REFERENCES users(id)
);