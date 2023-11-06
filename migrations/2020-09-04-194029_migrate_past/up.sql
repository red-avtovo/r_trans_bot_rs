create table if not exists users
(
    id         bigint       not null
        constraint users_pkey
            primary key,
    chat       bigint       not null
        constraint users_chat_key
            unique,
    first_name varchar(255) not null,
    last_name  varchar(255),
    username   varchar(255),
    salt       varchar(255) not null,
    created_at timestamp with time zone default CURRENT_TIMESTAMP
);

create table if not exists dirs
(
    id         uuid         not null
        constraint dirs_pkey
            primary key,
    user_id    bigint       not null
        constraint dirs_user_id_fkey
            references users
            on update restrict on delete restrict,
    alias      varchar(100) not null,
    path       varchar(255) not null,
    ordinal    integer      not null,
    created_at timestamp with time zone default CURRENT_TIMESTAMP
);

create table if not exists servers
(
    id         uuid         not null
        constraint servers_pkey
            primary key,
    user_id    bigint       not null
        constraint servers_user_id_fkey
            references users
            on update restrict on delete restrict,
    url        varchar(255) not null,
    username   varchar(255),
    password   varchar(255),
    created_at timestamp with time zone default CURRENT_TIMESTAMP
);

create table if not exists magnets
(
    id         uuid          not null
        constraint magnets_pkey
            primary key,
    user_id    bigint        not null
        constraint magnets_user_id_fkey
            references users
            on update restrict on delete restrict,
    url        varchar(4096) not null,
    created_at timestamp with time zone default CURRENT_TIMESTAMP
);

create table if not exists tasks
(
    id          uuid        not null
        constraint tasks_pkey
            primary key,
    user_id     bigint      not null
        constraint tasks_user_id_fkey
            references users
            on update restrict on delete restrict,
    server_id   uuid        not null
        constraint tasks_server_id_fkey
            references servers
            on update restrict on delete cascade,
    magnet_id   uuid        not null
        constraint tasks_magnet_id_fkey
            references magnets
            on update restrict on delete restrict,
    status      varchar(50) not null,
    description text,
    created_at  timestamp with time zone default CURRENT_TIMESTAMP
);