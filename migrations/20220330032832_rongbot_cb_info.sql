-- Adds channel type for new rong permissions.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'channel_persona') THEN
        CREATE TYPE channel_persona AS
            ENUM ('cb', 'clan', 'pvp', 'public');
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS rongbot.channel_type(
    channel_id character varying(20) NOT NULL,
    clan_id integer REFERENCES rong_clan NOT NULL,
    persona channel_persona NOT NULL,
    PRIMARY KEY (channel_id, clan_id)
);

ALTER TABLE rongbot.channel_type OWNER TO rongprod;
