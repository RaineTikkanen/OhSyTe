PRAGMA foreign_keys = ON;

INSERT INTO category VALUES (1, 'test', NULL);
INSERT INTO category VALUES (100, 'history', NULL);

INSERT INTO event (event_date, event_description, category_id)
VALUES ('2026-01-26', 'Test event #1 for Jan 26, 2026', 1);
INSERT INTO event (event_date, event_description, category_id)
VALUES ('2026-01-26', 'Test event #2 for Jan 26, 2026', 1);

INSERT INTO event (event_date, event_description, category_id)
VALUES ('2003-19-03', 'The United States launches Operation Iraqi Freedom, beginning the invasion of Iraq.', 100);
INSERT INTO event (event_date, event_description, category_id)
VALUES ('1937-19-03', 'Astronomer Fritz Zwicky publishes his research on stellar explosions, coining the term "supernova" and hypothesizing that they are the origin of cosmic rays.', 100);
INSERT INTO event (event_date, event_description, category_id)
VALUES ('1977-19-03', 'France performs nuclear test at Mururoa Atoll', 100);
INSERT INTO event (event_date, event_description, category_id)
VALUES ('1982-19-03', 'Falklands War: Argentinian forces land on South Georgia Island, precipitating war with the U.K.', 100);
INSERT INTO event (event_date, event_description, category_id)
VALUES ('2013-19-03', 'NASA''s Mars rover Curiosity discovers further evidence of water-bearing minerals', 100);