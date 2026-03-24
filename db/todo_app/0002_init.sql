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

create table if not exists auth.users (
    id uuid primary key default gen_random_uuid(),
    username text not null unique
        check (username = lower(username))
        check (username ~ '^[a-z0-9_]{3,32}$'),
    password_hash text not null,
    created_at timestamptz not null default now()
);

create table if not exists public.todos (
    id uuid primary key default gen_random_uuid(),
    owner_id uuid not null references auth.users(id) on delete cascade,
    title text not null check (length(trim(title)) > 0),
    description text not null default '',
    is_completed boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index if not exists auth_users_username_idx
    on auth.users (username);

create index if not exists todos_owner_id_created_at_idx
    on public.todos (owner_id, created_at desc);

drop trigger if exists todos_touch_updated_at on public.todos;

create trigger todos_touch_updated_at
before update on public.todos
for each row
execute function public.touch_updated_at();

create or replace function auth.register_user(
    p_username text,
    p_password text
)
returns auth.users
language plpgsql
security definer
set search_path = auth, public
as $$
declare
    new_user auth.users;
begin
    if p_username is null or length(trim(p_username)) = 0 then
        raise exception 'username is required';
    end if;

    if p_password is null or length(p_password) < 6 then
        raise exception 'password must be at least 6 characters';
    end if;

    insert into auth.users (username, password_hash)
    values (
        lower(trim(p_username)),
        crypt(p_password, gen_salt('bf'))
    )
    returning * into new_user;

    return new_user;
exception
    when unique_violation then
        raise exception 'username already exists';
end;
$$;

create or replace function auth.login_user(
    p_username text,
    p_password text
)
returns auth.users
language plpgsql
security definer
set search_path = auth, public
as $$
declare
    existing_user auth.users;
begin
    select *
    into existing_user
    from auth.users
    where username = lower(trim(p_username));

    if existing_user.id is null then
        raise exception 'invalid username or password';
    end if;

    if existing_user.password_hash <> crypt(p_password, existing_user.password_hash) then
        raise exception 'invalid username or password';
    end if;

    return existing_user;
end;
$$;

alter table public.todos enable row level security;

drop policy if exists todos_select_own on public.todos;
drop policy if exists todos_insert_own on public.todos;
drop policy if exists todos_update_own on public.todos;
drop policy if exists todos_delete_own on public.todos;

create policy todos_select_own
on public.todos
for select
using (
    owner_id = public.current_app_user_id()
);

create policy todos_insert_own
on public.todos
for insert
with check (
    owner_id = public.current_app_user_id()
);

create policy todos_update_own
on public.todos
for update
using (
    owner_id = public.current_app_user_id()
)
with check (
    owner_id = public.current_app_user_id()
);

create policy todos_delete_own
on public.todos
for delete
using (
    owner_id = public.current_app_user_id()
);

grant usage on schema auth to manhblue;
grant usage on schema public to manhblue;
grant select, insert, update, delete on auth.users to manhblue;
grant select, insert, update, delete on public.todos to manhblue;
grant execute on function auth.register_user(text, text) to manhblue;
grant execute on function auth.login_user(text, text) to manhblue;
