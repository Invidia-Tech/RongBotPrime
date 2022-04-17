-- This migration adds the rong tables used for ATC (Air Traffic Control)

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'flight_status') THEN
        CREATE TYPE flight_status AS
            ENUM ('canceled', 'in flight', 'landed', 'crashed', 'amb');
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS rongbot.pilot(
    id SERIAL PRIMARY KEY NOT NULL,
    nickname character varying(40),
    motto text,
    code character varying(10),
    clan_id integer REFERENCES rong_clan NOT NULL,
    user_id integer REFERENCES rong_user NOT NULL,
    UNIQUE (clan_id, user_id)
);

ALTER TABLE rongbot.pilot OWNER TO rongprod;


CREATE TABLE IF NOT EXISTS rongbot.flight(
    id SERIAL PRIMARY KEY NOT NULL,
    call_sign character varying(20) NOT NULL,
    pilot_id integer REFERENCES rongbot.pilot NOT NULL,
    clan_id integer REFERENCES rong_clan NOT NULL,
    cb_id integer REFERENCES rong_clanbattle NOT NULL,
    passenger_id integer REFERENCES rong_clanmember,
    start_time timestamptz NOT NULL,
    end_time timestamptz,
    status flight_status NOT NULL,
    team_id integer REFERENCES rong_team -- optional
);

ALTER TABLE rongbot.flight OWNER TO rongprod;


CREATE TABLE IF NOT EXISTS rongbot.flight_metadata(
    clan_id integer REFERENCES rong_clan NOT NULL,
    cb_id integer REFERENCES rong_clanbattle NOT NULL,
    call_sign_prefix character varying(10) NOT NULL,
    PRIMARY KEY (clan_id, cb_id)
);

ALTER TABLE rongbot.flight_metadata OWNER TO rongprod;
