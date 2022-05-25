-- Force quits table

DROP TABLE IF EXISTS rongbot.force_quits;

CREATE TABLE IF NOT EXISTS rongbot.force_quit(
    clanmember_id integer REFERENCES rong_clanmember NOT NULL,
    cb_id integer REFERENCES rong_clanbattle NOT NULL,
    time_used timestamptz NOT NULL,
    day_used integer NOT NULL,
    note TEXT,
    PRIMARY KEY (clanmember_id, cb_id, day_used)
);

ALTER TABLE rongbot.force_quit OWNER TO rongprod;
