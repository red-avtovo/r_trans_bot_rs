create table if not exists friends(
    id uuid not null default gen_random_uuid() primary key,
    user_id bigint not null references users
    on update restrict
    on delete restrict,
    friend_user_id bigint not null references users
    on update restrict
    on delete restrict,
    created_at timestamp  with time zone not null default CURRENT_TIMESTAMP
);