-- Chat app bootstrap schema for a separate PostgreSQL database.
-- Replace the grantee role below if you do not run the API as `manhblue`.

create extension if not exists pgcrypto;

create schema if not exists auth;

create or replace function public.current_app_user_id()
returns uuid
language sql
stable
as $$
    select nullif(current_setting('request.jwt.claim.sub', true), '')::uuid
$$;

create or replace function public.touch_updated_at()
returns trigger
language plpgsql
as $$
begin
    new.updated_at = now();
    return new;
end;
$$;

create or replace function public.sync_profile_from_user()
returns trigger
language plpgsql
as $$
begin
    insert into public.profiles (user_id, username)
    values (new.id, new.username)
    on conflict (user_id) do update
    set username = excluded.username;

    return new;
end;
$$;

create or replace function public.lock_profile_identity_fields()
returns trigger
language plpgsql
as $$
begin
    new.user_id = old.user_id;
    new.username = old.username;
    return new;
end;
$$;

create table if not exists auth.users (
    id uuid primary key default gen_random_uuid(),
    username text not null unique
        check (username = lower(username))
        check (username ~ '^[a-z0-9_]{3,32}$'),
    password_hash text not null,
    created_at timestamptz not null default now()
);

create table if not exists public.profiles (
    user_id uuid primary key references auth.users(id) on delete cascade,
    username text not null unique
        check (username = lower(username))
        check (username ~ '^[a-z0-9_]{3,32}$'),
    display_name text,
    avatar_url text,
    bio text,
    updated_at timestamptz not null default now()
);

create table if not exists public.contact_requests (
    id uuid primary key default gen_random_uuid(),
    from_user uuid not null references auth.users(id) on delete cascade,
    to_user uuid not null references auth.users(id) on delete cascade,
    status text not null default 'pending'
        check (status in ('pending', 'accepted', 'rejected')),
    created_at timestamptz not null default now(),
    check (from_user <> to_user),
    unique (from_user, to_user)
);

create table if not exists public.contacts (
    user_id uuid not null references auth.users(id) on delete cascade,
    contact_id uuid not null references auth.users(id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, contact_id),
    check (user_id <> contact_id)
);

create index if not exists auth_users_username_idx
    on auth.users (username);

create unique index if not exists contact_requests_pending_pair_idx
    on public.contact_requests (
        least(from_user, to_user),
        greatest(from_user, to_user)
    )
    where status = 'pending';

create index if not exists contact_requests_to_user_idx
    on public.contact_requests (to_user, status, created_at desc);

create index if not exists contact_requests_from_user_idx
    on public.contact_requests (from_user, status, created_at desc);

create index if not exists contacts_contact_id_idx
    on public.contacts (contact_id);

drop trigger if exists profiles_touch_updated_at on public.profiles;
drop trigger if exists profiles_lock_identity_fields on public.profiles;
drop trigger if exists auth_users_sync_profile on auth.users;

create trigger auth_users_sync_profile
after insert or update of username on auth.users
for each row
execute function public.sync_profile_from_user();

create trigger profiles_touch_updated_at
before update on public.profiles
for each row
execute function public.touch_updated_at();

create trigger profiles_lock_identity_fields
before update on public.profiles
for each row
execute function public.lock_profile_identity_fields();

alter table public.profiles enable row level security;
alter table public.contact_requests enable row level security;
alter table public.contacts enable row level security;

drop policy if exists profiles_read_all on public.profiles;
drop policy if exists profiles_update_self on public.profiles;

create policy profiles_read_all
on public.profiles
for select
using (true);

create policy profiles_update_self
on public.profiles
for update
using (
    user_id = public.current_app_user_id()
)
with check (
    user_id = public.current_app_user_id()
);

drop policy if exists contact_requests_select_own on public.contact_requests;
drop policy if exists contact_requests_insert_self on public.contact_requests;
drop policy if exists contact_requests_update_receiver on public.contact_requests;
drop policy if exists contact_requests_delete_sender on public.contact_requests;

create policy contact_requests_select_own
on public.contact_requests
for select
using (
    from_user = public.current_app_user_id()
    or to_user = public.current_app_user_id()
);

create policy contact_requests_insert_self
on public.contact_requests
for insert
with check (
    from_user = public.current_app_user_id()
    and to_user <> public.current_app_user_id()
    and status = 'pending'
);

create policy contact_requests_update_receiver
on public.contact_requests
for update
using (
    to_user = public.current_app_user_id()
    and status = 'pending'
)
with check (
    to_user = public.current_app_user_id()
    and from_user <> public.current_app_user_id()
    and status in ('accepted', 'rejected')
);

create policy contact_requests_delete_sender
on public.contact_requests
for delete
using (
    from_user = public.current_app_user_id()
    and status = 'pending'
);

drop policy if exists contacts_select_own on public.contacts;
drop policy if exists contacts_delete_self on public.contacts;

create policy contacts_select_own
on public.contacts
for select
using (
    user_id = public.current_app_user_id()
);

create policy contacts_delete_self
on public.contacts
for delete
using (
    user_id = public.current_app_user_id()
);

create or replace function public.accept_contact_request(req_id uuid)
returns void
language plpgsql
security definer
set search_path = public, auth
as $$
declare
    me uuid := public.current_app_user_id();
    other uuid;
begin
    if me is null then
        raise exception 'unauthorized';
    end if;

    select from_user into other
    from public.contact_requests
    where id = req_id
      and to_user = me
      and status = 'pending'
    for update;

    if other is null then
        raise exception 'invalid request';
    end if;

    insert into public.contacts (user_id, contact_id)
    values (me, other), (other, me)
    on conflict do nothing;

    update public.contact_requests
    set status = 'accepted'
    where id = req_id;
end;
$$;

create or replace function public.reject_contact_request(req_id uuid)
returns void
language plpgsql
security definer
set search_path = public, auth
as $$
declare
    me uuid := public.current_app_user_id();
begin
    if me is null then
        raise exception 'unauthorized';
    end if;

    update public.contact_requests
    set status = 'rejected'
    where id = req_id
      and to_user = me
      and status = 'pending';

    if not found then
        raise exception 'invalid request';
    end if;
end;
$$;

create or replace function public.remove_contact(other_user uuid)
returns void
language plpgsql
security definer
set search_path = public, auth
as $$
declare
    me uuid := public.current_app_user_id();
begin
    if me is null then
        raise exception 'unauthorized';
    end if;

    delete from public.contacts
    where (user_id = me and contact_id = other_user)
       or (user_id = other_user and contact_id = me);
end;
$$;

grant usage on schema auth to manhblue;
grant usage on schema public to manhblue;

grant select, insert on auth.users to manhblue;

grant select, update on public.profiles to manhblue;
grant select, insert, update, delete on public.contact_requests to manhblue;
grant select, delete on public.contacts to manhblue;

grant execute on function public.accept_contact_request(uuid) to manhblue;
grant execute on function public.reject_contact_request(uuid) to manhblue;
grant execute on function public.remove_contact(uuid) to manhblue;
