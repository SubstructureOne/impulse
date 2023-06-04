-- ensure postgres usernames of provisioned users are unique
CREATE UNIQUE INDEX users_pgname_ix ON users (pg_name);
