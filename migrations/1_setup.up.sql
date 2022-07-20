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

CREATE ROLE natter_api_user WITH LOGIN PASSWORD 'password';
GRANT SELECT, INSERT, UPDATE ON spaces, messages TO natter_api_user;

CREATE TABLE users (
    user_id VARCHAR(30) PRIMARY KEY,
    pw_hash VARCHAR(255) NOT NULL
);
GRANT SELECT, INSERT ON users TO natter_api_user;

CREATE TABLE audit_log (
    audit_id BIGINT NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(100) NOT NULL,
    user_id VARCHAR(30) NULL,
    status INT NULL,
    audit_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE SEQUENCE audit_id_seq;
GRANT SELECT, INSERT ON audit_log TO natter_api_user;




GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO natter_api_user;