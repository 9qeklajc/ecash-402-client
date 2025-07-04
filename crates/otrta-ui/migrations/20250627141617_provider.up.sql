-- Add up migration script here

CREATE TABLE providers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL UNIQUE,
    mints TEXT[] DEFAULT '{}',
    use_onion BOOLEAN DEFAULT FALSE,
    followers INTEGER DEFAULT 0,
    zaps INTEGER DEFAULT 0,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert some default test data
INSERT INTO providers (name, url, mints, use_onion, followers, zaps, is_default) VALUES
('otrta ai', 'https://ecash.otrta.me', ARRAY['https://ecashmint.otrta.me'], false, 0, 0, TRUE)

