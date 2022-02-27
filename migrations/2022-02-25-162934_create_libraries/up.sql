CREATE TABLE public.libraries
(
    id VARCHAR PRIMARY KEY NOT NULL,
    path VARCHAR NOT NULL,
    depth INTEGER NOT NULL,
    last_scan INTEGER NOT NULL
)

TABLESPACE pg_default;

ALTER TABLE public.libraries
    OWNER to postgres;