CREATE TABLE public.files
(
    id SERIAL PRIMARY KEY NOT NULL,
    parent VARCHAR NOT NULL,
    library_id VARCHAR NOT NULL,
    path VARCHAR NOT NULL,
    folder BOOLEAN NOT NULL,
    last_update BIGINT NOT NULL,
    title VARCHAR,
    season VARCHAR,
    episode REAL,
    release_group VARCHAR
)

TABLESPACE pg_default;

ALTER TABLE public.files
    OWNER to postgres;