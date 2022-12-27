CREATE TYPE chargetype AS ENUM (
    'DataTransferInBytes',
    'DataTransferOutBytes',
    'DataStorageByteHours'
);

CREATE TYPE timechargetype AS ENUM (
    'DataStorageBytes'
);

CREATE TABLE charges (
    charge_id bigserial PRIMARY KEY,
    charge_time timestamptz NOT NULL DEFAULT current_timestamp,
    user_id uuid NOT NULL,
    charge_type chargetype NOT NULL,
    quantity double precision NOT NULL,
    rate double precision NOT NULL,
    amount double precision NOT NULL GENERATED ALWAYS AS (quantity * rate) STORED
);
CREATE INDEX charges_user_index on charges(user_id, charge_time ASC);

CREATE TABLE timecharges(
    timecharge_id bigserial PRIMARY KEY,
    timecharge_time timestamptz NOT NULL DEFAULT current_timestamp,
    user_id uuid NOT NULL,
    timecharge_type timechargetype NOT NULL,
    quantity double precision NOT NULL
);
CREATE INDEX timecharges_user_index on timecharges(user_id, timecharge_time ASC);

CREATE TABLE balances(
    user_id uuid PRIMARY KEY,
    balance double precision NOT NULL default 0.,
    created_at timestamptz NOT NULL default current_timestamp,
    updated_at timestamptz NOT NULL default current_timestamp
);
SELECT diesel_manage_updated_at('balances');

CREATE TABLE exttransactions(
    exttransaction_id bigserial PRIMARY KEY,
    user_id uuid NOT NULL,
    amount double precision NOT NULL,
    exttransaction_time timestamptz NOT NULL default current_timestamp
);
CREATE INDEX exttransactions_user_index on exttransactions(user_id);
