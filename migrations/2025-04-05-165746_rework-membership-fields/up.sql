-- Purge the database before adding a NOT NULL field
DELETE FROM membership;
DELETE FROM last_update WHERE element = 'Memberships';

ALTER TABLE membership ADD COLUMN cell_number VARCHAR;
ALTER TABLE membership ADD COLUMN start_date VARCHAR NOT NULL default '';

ALTER TABLE membership DROP COLUMN gender;
ALTER TABLE membership DROP COLUMN age;
ALTER TABLE membership DROP COLUMN payed;
ALTER TABLE membership DROP COLUMN expired;
