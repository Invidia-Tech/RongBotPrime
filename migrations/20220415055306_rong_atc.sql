-- This migration adds the rong tables used for ATC (Air Traffic Control)

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'flight_status') THEN
        CREATE TYPE flight_status AS
            ENUM ('canceled', 'in flight', 'landed', 'crashed', 'amb');
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS rongbot.pilot(
    pilot_id integer PRIMARY KEY NOT NULL,
    nickname character varying(40),
    motto text,
    code character varying(10),
    clan_id integer REFERENCES rong_clan NOT NULL,
    user_id integer REFERENCES rong_user NOT NULL,
    UNIQUE (clan_id, user_id)
);

ALTER TABLE rongbot.pilot OWNER TO rongprod;


CREATE TABLE IF NOT EXISTS rongbot.flight(
    flight_id integer PRIMARY KEY NOT NULL,
    call_sign character varying(20),
    pilot_id integer REFERENCES rongbot.pilot NOT NULL,
    clan_id integer REFERENCES rong_clan NOT NULL,
    cb_id integer REFERENCES rong_clanbattle NOT NULL,
    passenger_id integer REFERENCES rong_clanmember NOT NULL,
    start_time timestamptz NOT NULL,
    end_time timestamptz,
    status flight_status NOT NULL,
    team_id integer REFERENCES rong_team -- optional
);

ALTER TABLE rongbot.flight OWNER TO rongprod;
