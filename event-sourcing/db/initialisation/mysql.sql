CREATE SCHEMA es;

USE es;

CREATE TABLE events
(
    id                 VARBINARY(16)   NOT NULL,
    aggregate_name     VARCHAR(50)     NOT NULL,
    aggregate_id       VARBINARY(16)   NOT NULL,
    aggregate_sequence BIGINT UNSIGNED NOT NULL,
    event_name         VARCHAR(50)     NOT NULL,
    event_version      VARCHAR(10)     NOT NULL,
    event_payload      TEXT            NOT NULL,
    metadata           TEXT            NOT NULL
);

CREATE UNIQUE INDEX ix01_events ON events (aggregate_name, aggregate_id, aggregate_sequence);
