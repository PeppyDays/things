CREATE SCHEMA IF NOT EXISTS account;

USE account;

-- event sourcing table for user aggregate now
CREATE TABLE events (
    id                 VARBINARY(16) NOT NULL,
    aggregate_name     VARCHAR(50)   NOT NULL,
    aggregate_id       VARBINARY(16) NOT NULL,
    aggregate_sequence BIGINT        NOT NULL,
    event_name         VARCHAR(50)   NOT NULL,
    event_version      VARCHAR(10)   NOT NULL,
    event_payload      JSON          NOT NULL,
    metadata           JSON          NOT NULL,
    created_at         DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at        DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX ix01_events ON events (aggregate_name, aggregate_id, aggregate_sequence);

-- identity aggregate table
CREATE TABLE identities (
    user_id       VARBINARY(16) NOT NULL,
    user_role     VARCHAR(10)   NOT NULL,
    refresh_token VARCHAR(256)  NULL,
    created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at   DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX ix01_identities ON identities (user_id);
