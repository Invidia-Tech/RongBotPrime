-- Add new metadata field and force close table

CREATE TABLE IF NOT EXISTS rongbot.atc_config(
    clan_id integer PRIMARY KEY REFERENCES rong_clan NOT NULL,
    alert_channel character varying(20),
    auto_alert_duration TEXT
);

ALTER TABLE rongbot.atc_config OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.force_quits(
    clanmember_id integer REFERENCES rong_clanmember NOT NULL,
    cb_id integer REFERENCES rong_clanbattle NOT NULL,
    day_used integer,
    used_amount integer DEFAULT 0,
    PRIMARY KEY (clanmember_id, cb_id)
);

ALTER TABLE rongbot.force_quits OWNER TO rongprod;
