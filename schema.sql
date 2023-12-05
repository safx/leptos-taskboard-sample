DROP TABLE IF EXISTS Tasks;
CREATE TABLE IF NOT EXISTS Tasks (id VARCHAR(36) PRIMARY KEY, name TEXT, assignee TEXT, mandays INTEGER, status INTEGER);
INSERT INTO Tasks (id, name, assignee, mandays, status) VALUES ('00000000-0000-0000-0000-000000000000', 'My first task', 'Admin', 1, 0);
