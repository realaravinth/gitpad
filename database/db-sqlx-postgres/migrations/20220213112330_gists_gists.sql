CREATE TABLE IF NOT EXISTS gists_visibility (
    name VARCHAR(15) NOT NULL UNIQUE,
	ID SERIAL PRIMARY KEY NOT NULL
);

INSERT INTO gists_visibility (name) VALUES('private') ON CONFLICT (name) DO NOTHING;
INSERT INTO gists_visibility (name) VALUES('unlisted') ON CONFLICT (name) DO NOTHING;
INSERT INTO gists_visibility (name) VALUES('public') ON CONFLICT (name) DO NOTHING;

CREATE TABLE IF NOT EXISTS gists_gists (
	owner_id INTEGER NOT NULL references gists_users(ID) ON DELETE CASCADE,
    visibility INTEGER NOT NULL references gists_visibility(ID),
	description TEXT DEFAULT NULL,
	created timestamptz NOT NULL,
	updated timestamptz NOT NULL,
	public_id VARCHAR(32) UNIQUE NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);

CREATE INDEX ON gists_gists(public_id);

CREATE TABLE IF NOT EXISTS gists_comments (
	owner_id INTEGER NOT NULL references gists_users(ID) ON DELETE CASCADE,
	gist_id INTEGER NOT NULL references gists_gists(ID) ON DELETE CASCADE,
	comment TEXT DEFAULT NULL,
	created timestamptz NOT NULL DEFAULT now(),
	ID SERIAL PRIMARY KEY NOT NULL
);
