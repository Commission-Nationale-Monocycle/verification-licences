-- Purge the database before adding a NOT NULL field
DELETE FROM membership;
DELETE FROM last_update WHERE element = 'Memberships';

ALTER TABLE membership
    DROP COLUMN cell_number;
ALTER TABLE membership
    DROP COLUMN start_date;

ALTER TABLE membership ADD COLUMN gender VARCHAR NOT NULL DEFAULT '';
ALTER TABLE membership ADD COLUMN age INTEGER;
ALTER TABLE membership ADD COLUMN payed BOOLEAN NOT NULL DEFAULT '';
ALTER TABLE membership ADD COLUMN expired BOOLEAN NOT NULL DEFAULT '';
