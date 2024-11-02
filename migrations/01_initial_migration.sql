-- Initial Migration Script here

CREATE TABLE watching (
	message_id INTEGER NOT NULL primary key,
	channel_id INTEGER NOT NULL,
	cache_score FLOAT,
	linked_user INTEGER,
	FOREIGN KEY(linked_user) REFERENCES users(user_id)
);

CREATE TABLE users (
	user_id INTEGER NOT NULL primary key,
	cached_score FLOAT
)
