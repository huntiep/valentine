UPDATE users SET num_repos = num_repos + 1
    WHERE username = $1;
INSERT INTO repos VALUES ($2, $3, $1);
