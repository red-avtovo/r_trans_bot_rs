create table r_users
(
    id         bigint       not null
        constraint r_users_pkey
            primary key,
    chat       bigint       not null
        constraint r_users_chat_key unique,
    first_name varchar(255) not null,
    last_name  varchar(255),
    username   varchar(255),
    salt       varchar(255) not null,
    created_at timestamp with time zone default CURRENT_TIMESTAMP
);