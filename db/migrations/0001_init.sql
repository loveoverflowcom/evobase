create extension if not exists pgcrypto;

create schema if not exists auth;

create table if not exists auth.users (
    id uuid primary key default gen_random_uuid(),
    username text not null unique,
    password_hash text not null,
    created_at timestamptz not null default now()
);

create table if not exists public.notes (
    id uuid primary key default gen_random_uuid(),
    owner_id uuid not null references auth.users(id) on delete cascade,
    body text not null,
    created_at timestamptz not null default now()
);

alter table public.notes enable row level security;

drop policy if exists notes_owner_policy on public.notes;

create policy notes_owner_policy
on public.notes
using (
    owner_id = nullif(current_setting('request.jwt.claim.sub', true), '')::uuid
)
with check (
    owner_id = nullif(current_setting('request.jwt.claim.sub', true), '')::uuid
);

grant usage on schema auth to postgres;
grant usage on schema public to postgres;
grant select, insert, update, delete on auth.users to postgres;
grant select, insert, update, delete on public.notes to postgres;
