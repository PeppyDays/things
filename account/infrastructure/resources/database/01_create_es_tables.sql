CREATE SCHEMA account;

USE account;

CREATE TABLE events (
    id                 VARBINARY(16) NOT NULL,
    aggregate_name     VARCHAR(50)   NOT NULL,
    aggregate_id       VARBINARY(16) NOT NULL,
    aggregate_sequence BIGINT        NOT NULL,
    event_name         VARCHAR(50)   NOT NULL,
    event_version      VARCHAR(10)   NOT NULL,
    event_payload      JSON          NOT NULL,
    metadata           JSON          NOT NULL
);

CREATE UNIQUE INDEX ix01_events ON events (aggregate_name, aggregate_id, aggregate_sequence);

CREATE TABLE identities (
    user_id   VARBINARY(16) NOT NULL,
    user_role VARCHAR(10)   NOT NULL,
    refresh_token VARCHAR(256) NOT NULL
);

CREATE UNIQUE INDEX ix01_identities ON identities (user_id, user_role);
