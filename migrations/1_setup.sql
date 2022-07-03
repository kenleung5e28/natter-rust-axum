CREATE TABLE IF NOT EXISTS spaces (
    space_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(30) NOT NULL
);
CREATE TABLE IF NOT EXISTS messages (
    space_id INT NOT NULL REFERENCES spaces(space_id),
    msg_id SERIAL PRIMARY KEY,
    author VARCHAR(30) NOT NULL,
    msg_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    msg_text VARCHAR(1024) NOT NULL
);
CREATE INDEX IF NOT EXISTS msg_timestamp_idx ON messages(msg_time);
CREATE UNIQUE INDEX IF NOT EXISTS space_name_idx ON spaces(name);
