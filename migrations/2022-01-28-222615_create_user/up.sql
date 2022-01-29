CREATE TABLE public.users
(
    id VARCHAR PRIMARY KEY NOT NULL,
    username VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    permissions TEXT[] NOT NULL
)

TABLESPACE pg_default;

ALTER TABLE public.users
    OWNER to postgres;