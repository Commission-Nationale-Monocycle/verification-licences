ALTER TABLE membership
    RENAME COLUMN normalized_membership_number TO old_normalized_membership_number;
ALTER TABLE membership
    RENAME COLUMN normalized_last_name TO old_normalized_last_name;
ALTER TABLE membership
    RENAME COLUMN normalized_first_name TO old_normalized_first_name;
ALTER TABLE membership
    RENAME COLUMN normalized_last_name_first_name TO old_normalized_last_name_first_name;
ALTER TABLE membership
    RENAME COLUMN normalized_first_name_last_name TO old_normalized_first_name_last_name;

ALTER TABLE membership
    ADD COLUMN normalized_membership_number VARCHAR NOT NULL DEFAULT 0;
ALTER TABLE membership
    ADD COLUMN normalized_last_name VARCHAR NOT NULL DEFAULT 0;
ALTER TABLE membership
    ADD COLUMN normalized_first_name VARCHAR NOT NULL DEFAULT 0;
ALTER TABLE membership
    ADD COLUMN normalized_last_name_first_name VARCHAR NOT NULL DEFAULT 0;
ALTER TABLE membership
    ADD COLUMN normalized_first_name_last_name VARCHAR NOT NULL DEFAULT 0;

UPDATE membership
SET (normalized_membership_number, normalized_last_name, normalized_first_name, normalized_last_name_first_name, normalized_first_name_last_name) =
        (old_normalized_membership_number, old_normalized_last_name, old_normalized_first_name,
         old_normalized_last_name_first_name, old_normalized_first_name_last_name);

CREATE INDEX normalized_membership_number_index ON membership (normalized_membership_number);
CREATE INDEX normalized_last_name_index ON membership (normalized_last_name);
CREATE INDEX normalized_first_name_index ON membership (normalized_first_name);
CREATE INDEX normalized_last_name_first_name_index ON membership (normalized_last_name_first_name);
CREATE INDEX normalized_first_name_last_name_index ON membership (normalized_first_name_last_name);

ALTER TABLE membership DROP COLUMN old_normalized_membership_number;
ALTER TABLE membership DROP COLUMN old_normalized_last_name;
ALTER TABLE membership DROP COLUMN old_normalized_first_name;
ALTER TABLE membership DROP COLUMN old_normalized_last_name_first_name;
ALTER TABLE membership DROP COLUMN old_normalized_first_name_last_name;