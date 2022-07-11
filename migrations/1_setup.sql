DROP ROLE IF EXISTS natter_api_user;
DROP INDEX IF EXISTS msg_timestamp_idx;
DROP INDEX IF EXISTS space_name_idx;
DROP TABLE IF EXISTS spaces;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS users;

CREATE TABLE spaces (
    space_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(30) NOT NULL
);
CREATE TABLE messages (
    space_id INT NOT NULL REFERENCES spaces(space_id),
    msg_id SERIAL PRIMARY KEY,
    author VARCHAR(30) NOT NULL,
    msg_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    msg_text VARCHAR(1024) NOT NULL
);
CREATE INDEX msg_timestamp_idx ON messages(msg_time);
CREATE UNIQUE INDEX space_name_idx ON spaces(name);

CREATE ROLE natter_api_user PASSWORD 'password';
GRANT SELECT, INSERT ON spaces, messages TO natter_api_user;

CREATE TABLE users (
    user_id VARCHAR(30) PRIMARY KEY,
    pw_hash VARCHAR(255) NOT NULL
);
