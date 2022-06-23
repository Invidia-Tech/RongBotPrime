-- Adds ping tables, >ping >ping

CREATE TABLE IF NOT EXISTS rongbot.ping_rarity(
    server character varying(20) NOT NULL,
    rank integer NOT NULL,
    weight integer NOT NULL,
    PRIMARY KEY (server, rank)
);

ALTER TABLE rongbot.ping_rarity OWNER to rongprod;

-- INSERT INTO rongbot.ping_rarity (rank, weight) VALUES (1, 51) ON CONFLICT DO NOTHING;
-- INSERT INTO rongbot.ping_rarity (rank, weight) VALUES (2, 35) ON CONFLICT DO NOTHING;
-- INSERT INTO rongbot.ping_rarity (rank, weight) VALUES (3, 10) ON CONFLICT DO NOTHING;
-- INSERT INTO rongbot.ping_rarity (rank, weight) VALUES (4, 3) ON CONFLICT DO NOTHING;
-- INSERT INTO rongbot.ping_rarity (rank, weight) VALUES (5, 1) ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS rongbot.ping_droptable(
    id SERIAL PRIMARY KEY NOT NULL,
    server character varying(20) NOT NULL,
    user_id character varying(20) NOT NULL,
    rarity_rank integer NOT NULL,
    weight integer NOT NULL,
    nickname TEXT,
    flavor_text TEXT,
    UNIQUE (server, user_id)
);

ALTER TABLE rongbot.ping_droptable OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.ping_log(
    id SERIAL PRIMARY KEY NOT NULL,
    server character varying(20) NOT NULL,
    rolled_by character varying(20) NOT NULL,
    loot character varying(20) NOT NULL,
    rarity_rank integer NOT NULL,
    dropped_on timestamptz NOT NULL,
    nickname TEXT
);

ALTER TABLE rongbot.ping_log OWNER TO rongprod;
