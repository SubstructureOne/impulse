CREATE OR REPLACE FUNCTION add_external_deposit(
    IN to_user uuid,
    IN amount double precision,
    IN exttxn_extid text,
    OUT new_balance double precision
)
    LANGUAGE plpgsql
AS $BODY$
BEGIN
    IF amount < 0 THEN
        RAISE EXCEPTION 'Deposit amount must be non-negative: %', amount;
    END IF;
    INSERT INTO exttransactions (user_id, amount, exttransaction_extid)
    VALUES (to_user, amount, exttxn_extid);
    UPDATE users
    SET balance = balance + amount
    WHERE user_id = to_user
    RETURNING balance INTO new_balance;
END;
$BODY$;
