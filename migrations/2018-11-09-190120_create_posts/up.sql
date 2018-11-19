-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email VARCHAR,
  viber_id TEXT NOT NULL,
  broadcast BOOLEAN NOT NULL DEFAULT 'f'
)