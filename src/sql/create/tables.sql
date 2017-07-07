CREATE TABLE IF NOT EXISTS users (
    uuid UUID PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    num_repos BIGINT NOT NULL,
    is_admin BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS public_key (
    id SERIAL,
    owner UUID REFERENCES users (uuid) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    fingerprint VARCHAR NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repos (
    uuid UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    owner UUID REFERENCES users (uuid) ON DELETE CASCADE,
    private BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS issues (
    repo UUID REFERENCES repos (uuid) ON DELETE CASCADE,
    id BIGINT NOT NULL,
    parent BIGINT NOT NULL,
    subject VARCHAR,
    content TEXT NOT NULL,
    created TIMESTAMP,
    PRIMARY KEY (repo, id)
);
