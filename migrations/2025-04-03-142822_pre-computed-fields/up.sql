ALTER TABLE membership
    ADD COLUMN normalized_membership_number VARCHAR;
ALTER TABLE membership
    ADD COLUMN normalized_last_name VARCHAR;
ALTER TABLE membership
    ADD COLUMN normalized_first_name VARCHAR;
ALTER TABLE membership
    ADD COLUMN normalized_last_name_first_name VARCHAR;
ALTER TABLE membership
    ADD COLUMN normalized_first_name_last_name VARCHAR;