CREATE OR REPLACE FUNCTION create_pg_user(p_username text, p_password text)
    RETURNS void
    LANGUAGE plpgsql STRICT VOLATILE
AS $BODY$
BEGIN
    EXECUTE FORMAT('CREATE ROLE %I WITH LOGIN CREATEDB NOSUPERUSER NOINHERIT NOCREATEROLE PASSWORD %L', p_username, p_password);
end
$BODY$;

CREATE OR REPLACE FUNCTION drop_pg_user(p_username text)
    RETURNS void
    LANGUAGE plpgsql STRICT VOLATILE
AS $BODY$
BEGIN
    EXECUTE FORMAT('DROP ROLE IF EXISTS %I', p_username);
end;
$BODY$;

CREATE TYPE userstatus AS ENUM (
    'Active',
    'Disabled',
    'Deleted'
);

CREATE TABLE users (
    user_id uuid PRIMARY KEY,
    pg_name TEXT NOT NULL,
    user_status userstatus NOT NULL DEFAULT 'Active',
    balance double precision NOT NULL DEFAULT 0,
    status_synced bool NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NOT NULL DEFAULT current_timestamp
);
SELECT diesel_manage_updated_at('users');

CREATE OR REPLACE FUNCTION add_external_deposit(
    IN to_user uuid,
    IN amount double precision,
    OUT new_balance double precision
)
    LANGUAGE plpgsql
AS $BODY$
BEGIN
    IF amount < 0 THEN
        RAISE EXCEPTION 'Deposit amount must be non-negative: %', amount;
    END IF;
    INSERT INTO exttransactions (user_id, amount)
        VALUES (to_user, amount);
    UPDATE users
        SET balance = balance + amount
        WHERE user_id = to_user
        RETURNING balance INTO new_balance;
END;
$BODY$;


CREATE OR REPLACE FUNCTION add_internal_transaction(
    IN from_user uuid,
    IN to_user uuid,
    IN amount double precision,
    IN disable_at double precision,
    OUT from_user_balance double precision,
    OUT to_user_balance double precision
)
    LANGUAGE plpgsql
AS $$
BEGIN
    IF amount < 0 THEN
        RAISE EXCEPTION 'Transaction amount must be non-negative: %', amount;
    END IF;
    INSERT INTO transactions (from_user, to_user, amount)
        VALUES (from_user, to_user, amount);
    UPDATE users
        SET balance = balance - amount
        WHERE user_id = from_user
        RETURNING balance INTO from_user_balance;
    UPDATE users
        SET balance = balance + amount
        WHERE user_id = to_user
        RETURNING balance INTO to_user_balance;
    IF from_user_balance < disable_at THEN
        UPDATE users SET user_status = 'Disabled' WHERE user_id = from_user;
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION add_internal_transaction_from_reports(
    p_from_user uuid,
    p_to_user uuid,
    p_charge_ids bigint[],
    p_disable_at double precision
)
    RETURNS bigint
    LANGUAGE plpgsql
AS $BODY$
DECLARE
    amount_transacted double precision;
    from_user_balance double precision;
    new_txn_id bigint;
--     new_txn transactions;
BEGIN
    SELECT SUM(amount) INTO STRICT amount_transacted FROM charges WHERE charge_id = ANY(p_charge_ids);
    INSERT INTO transactions (from_user, to_user, charge_ids, amount)
        VALUES (p_from_user, p_to_user, p_charge_ids, amount_transacted)
        RETURNING txn_id into new_txn_id;
    UPDATE charges SET transacted = true WHERE charge_id = ANY (p_charge_ids);
    UPDATE users
        SET balance = balance - amount_transacted
        WHERE user_id = p_from_user
        RETURNING balance INTO from_user_balance;
    UPDATE users
        SET balance = balance + amount_transacted
        WHERE user_id = p_to_user;
    IF from_user_balance < p_disable_at THEN
        UPDATE users SET user_status = 'Disabled' WHERE user_id = from_user;
    END IF;
    RETURN new_txn_id;
END;
$BODY$;

CREATE OR REPLACE VIEW reports_to_charge AS
    SELECT packet_id as report_id, user_id, packet_type, direction, length(packet_bytes) as num_bytes FROM reports r
    LEFT OUTER JOIN users u ON u.pg_name = r.username
    WHERE NOT CHARGED;
