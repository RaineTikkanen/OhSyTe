CREATE TABLE IF NOT EXISTS event(
	event_id INTEGER PRIMARY KEY,
	event_date DATE NOT NULL,
	event_description TEXT NOT NULL,
	category_id INTEGER NOT NULL,
	FOREIGN KEY (category_id) REFERENCES category(category_id)
);


CREATE TABLE IF NOT EXISTS category(
	category_id INTEGER PRIMARY KEY,
	primary_name TEXT NOT NULL,
	secondary_name TEXT
);
