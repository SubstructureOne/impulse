-- add a unique external reference to external transactions to prevent double-counting
ALTER TABLE exttransactions ADD COLUMN exttransaction_extid uuid;
UPDATE exttransactions SET exttransaction_extid = exttransaction_id WHERE TRUE;
ALTER TABLE exttransactions ALTER COLUMN exttransaction_extid SET NOT NULL;
CREATE UNIQUE INDEX exttransactions_extid_index ON exttransactions (exttransaction_extid);
