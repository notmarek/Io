CREATE TABLE public.libraries
(
    id SERIAL PRIMARY KEY NOT NULL,
    path VARCHAR NOT NULL,
    depth INTEGER NOT NULL
)

TABLESPACE pg_default;

ALTER TABLE public.libraries
    OWNER to postgres;