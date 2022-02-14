CREATE TABLE IF NOT EXISTS gists_privacy (
    name VARCHAR(15) NOT NULL UNIQUE,
	ID INTEGER PRIMARY KEY NOT NULL
);

INSERT OR IGNORE INTO gists_privacy (name) VALUES('private');
INSERT OR IGNORE INTO gists_privacy (name) VALUES('unlisted');
INSERT OR IGNORE INTO gists_privacy (name) VALUES('public');

CREATE TABLE IF NOT EXISTS gists_gists (
	owner_id INTEGER NOT NULL references gists_users(ID) ON DELETE CASCADE,
	description TEXT DEFAULT NULL,
	created INTEGER NOT NULL,
	updated INTEGER NOT NULL,
    privacy INTEGER NOT NULL references gists_privacy(ID),
	public_id VARCHAR(32) UNIQUE NOT NULL,
	ID INTEGER PRIMARY KEY NOT NULL
);


CREATE UNIQUE INDEX IF NOT EXISTS public_id_index ON gists_gists (public_id);

CREATE TABLE IF NOT EXISTS gists_comments (
	owner_id INTEGER NOT NULL references gists_users(ID) ON DELETE CASCADE,
	gist_id INTEGER NOT NULL references gists_gists(ID) ON DELETE CASCADE,
	comment TEXT DEFAULT NULL,
	created INTEGER NOT NULL,
	ID INTEGER PRIMARY KEY NOT NULL
);
