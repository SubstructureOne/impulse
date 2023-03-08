DROP VIEW reports_to_charge;

ALTER TABLE reports
    ALTER COLUMN packet_type TYPE text,
    ALTER COLUMN direction TYPE text;

DROP TYPE pgpkttype;
DROP TYPE pktdirection;

CREATE VIEW reports_to_charge AS
    SELECT packet_id as report_id, user_id, packet_type, direction, length(packet_bytes) as num_bytes
    FROM reports r
    LEFT OUTER JOIN users u ON u.pg_name = r.username
    WHERE NOT CHARGED;
