-- Add up migration script here
-- Creates bot specific schema and tables for RongBot

CREATE SCHEMA IF NOT EXISTS rongbot;

ALTER SCHEMA rongbot OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.unit_alias (
    unit_id integer REFERENCES redive_en.unit_data,
    unit_name text,
    PRIMARY KEY (unit_id, unit_name)
);

ALTER TABLE rongbot.unit_alias OWNER TO rongprod;

INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (101001, 'rat') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105201, 'rima') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (101701, 'waifu') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (104301, 'mako') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105701, 'dj') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (101301, 'nana') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (103201, 'laughinglady') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (103601, 'kyouka') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (104401, 'ilya') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (104401, 'iliya') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (104501, 'kuuka') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105801, 'peco') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105901, 'kok') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105901, 'kkr') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (106301, 'miss') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (100701, 'pudding') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (106001, 'kyaru') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107501, 'specorine') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107101, 'chris') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107501, 'speco') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107601, 'skokkoro') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107601, 'skok') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107701, 'ssuzume') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108001, 'smifuyu') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107801, 'skyaru') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107801, 'skaryl') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107601, 'skkr') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107601, 'skokk') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (102901, 'noz') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (105901, 'kokk') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107801, 'wetcat1') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (103601, 'KyoukaSmile') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (104001, 'Ring') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107901, 'stamaki') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (107901, 'stama') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108101, 'hshinobu') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108201, 'hmiyako') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108201, 'hpudding') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108301, 'hmisaki') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108401, 'xchika') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108501, 'xkurumi') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108601, 'xayane') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108901, 'nyrei') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108901, 'nrei') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108801, 'nyui') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108801, 'nyyui') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108701, 'nyori') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (108701, 'nyhiyori') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (109001, 'veriko') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (109101, 'vshizu') ON CONFLICT DO NOTHING;
INSERT INTO rongbot.unit_alias (unit_id, unit_name) VALUES (109101, 'vshizuru') ON CONFLICT DO NOTHING;

-- Discord related tables for the use by the web app
-- DROP TABLE rongbot.discord_role_members;
-- DROP TABLE rongbot.discord_server_roles;
-- DROP TABLE rongbot.discord_servers ;

CREATE TABLE IF NOT EXISTS rongbot.discord_servers (
    server_id character varying(20) PRIMARY KEY NOT NULL,
    name text
);

ALTER TABLE rongbot.discord_servers OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.discord_server_roles (
    server_id character varying(20) REFERENCES rongbot.discord_servers NOT NULL,
    role_id character varying(20) PRIMARY KEY NOT NULL,
    name text
);

ALTER TABLE rongbot.discord_server_roles OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.discord_members (
    member_id character varying(20) PRIMARY KEY NOT NULL,
    nickname character varying(40),
    username character varying(40) NOT NULL,
    discriminator integer NOT NULL
);

ALTER TABLE rongbot.discord_members OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.discord_role_members (
    server_id character varying(20) REFERENCES rongbot.discord_servers NOT NULL,
    role_id character varying(20) REFERENCES rongbot.discord_server_roles NOT NULL,
    member_id character varying(20) REFERENCES rongbot.discord_members NOT NULL,
    PRIMARY KEY (member_id, role_id)
);

ALTER TABLE rongbot.discord_role_members OWNER TO rongprod;

CREATE TABLE IF NOT EXISTS rongbot.unit_metadata (
    unit_id integer REFERENCES redive_en.unit_data PRIMARY KEY,
    prime_id integer NOT NULL
);

ALTER TABLE rongbot.unit_metadata OWNER TO rongprod;
