-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email VARCHAR UNIQUE,
  viber_id VARCHAR UNIQUE,
  broadcast BOOLEAN NOT NULL DEFAULT 'f'
)