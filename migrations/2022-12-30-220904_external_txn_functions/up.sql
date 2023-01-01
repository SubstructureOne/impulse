CREATE TABLE transactions (
    txn_id bigserial PRIMARY KEY,
    txn_time timestamptz NOT NULL DEFAULT current_timestamp,
    from_user uuid NOT NULL,
    to_user uuid NOT NULL,
    charge_ids bigint[],
    amount double precision NOT NULL
);

CREATE OR REPLACE FUNCTION add_external_deposit(to_user uuid, amount numeric)
    RETURNS void
    LANGUAGE plpgsql
AS $$
BEGIN
    IF amount < 0 THEN
        RAISE EXCEPTION 'Deposit amount must be non-negative: %', amount;
    END IF;
    INSERT
    INTO exttransactions (user_id, amount)
    VALUES (to_user, amount);
    LOOP
        -- first try to update an existing balance
        UPDATE balances
        SET balance = balance + amount
        WHERE user_id = to_user;
        IF FOUND THEN
            RETURN;
        END IF;
        -- no balance row found, try to insert a new one with "amount"
        -- as the total balance
        BEGIN
            INSERT INTO balances (user_id, balance)
            VALUES (to_user, amount);
            RETURN;
        EXCEPTION WHEN unique_violation THEN
        -- try again by looping back to update
        END;
    END LOOP;
END;
$$;


CREATE OR REPLACE FUNCTION add_internal_transaction(
    from_user uuid,
    to_user uuid,
    amount numeric
)
    RETURNS numeric
    LANGUAGE plpgsql
AS $$
DECLARE newbalance NUMERIC;
BEGIN
    IF amount < 0 THEN
        RAISE EXCEPTION 'Transaction amount must be non-negative: %', amount;
    END IF;
    INSERT
    INTO transactions (from_user, to_user, amount)
    VALUES (from_user, to_user, amount);
    -- update sending balance, ensuring it doesn't drop below 0
    UPDATE balances
    SET balance = balance - amount
    WHERE user_id = from_user
    RETURNING balance INTO newbalance;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Insufficient funds: 0';
    END IF;
    IF newbalance < 0 THEN
        RAISE EXCEPTION 'Insufficient funds: %', newbalance + amount;
    END IF;
    -- update the receiving balance
    LOOP
        UPDATE balances
        SET balance = balance + amount
        WHERE user_id = to_user;
        IF FOUND THEN
            RETURN newbalance;
        END IF;
        BEGIN
            INSERT INTO balances (user_id, balance)
            VALUES (to_user, amount);
            RETURN newbalance;
        EXCEPTION WHEN unique_violation THEN
        -- try again by looping back to update
        END;
    END LOOP;
END;
$$;

