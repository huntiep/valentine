CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    num_repos BIGINT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT f
);

CREATE TABLE IF NOT EXISTS public_keys (
    id SERIAL PRIMARY KEY,
    owner INTEGER REFERENCES users (id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    fingerprint VARCHAR NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repos (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    owner INTEGER REFERENCES users (id) ON DELETE CASCADE,
    private BOOLEAN NOT NULL,
    unique (name, owner)
);

CREATE TABLE IF NOT EXISTS issues (
    repo BIGINT REFERENCES repos (id) ON DELETE CASCADE,
    id BIGINT NOT NULL,
    parent BIGINT NOT NULL,
    subject VARCHAR,
    content TEXT NOT NULL,
    created TIMESTAMP NOT NULL,
    PRIMARY KEY (repo, id)
);