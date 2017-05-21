CREATE TABLE IF NOT EXISTS users (
    username VARCHAR PRIMARY KEY,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    num_repos BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS repos (
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    owner VARCHAR NOT NULL,
    PRIMARY KEY (name, owner)
);
