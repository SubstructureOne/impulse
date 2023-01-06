CREATE TABLE transactions (
    txn_id bigserial PRIMARY KEY,
    txn_time timestamptz NOT NULL DEFAULT current_timestamp,
    from_user uuid NOT NULL,
    to_user uuid NOT NULL,
    charge_ids bigint[] CHECK (array_position(charge_ids, NULL) IS NULL),
    amount double precision NOT NULL
);
