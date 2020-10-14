CREATE TYPE task_status AS ENUM ('created', 'started', 'finished', 'error');

CREATE table users
(
    id         BIGINT PRIMARY KEY,
    chat       BIGINT UNIQUE NOT NULL,
    first_name VARCHAR(255)  NOT NULL,
    last_name  VARCHAR(255),
    username   VARCHAR(255),
    salt       VARCHAR(255)  NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE table dirs
(
    id         UUID PRIMARY KEY,
    user_id    BIGINT       NOT NULL REFERENCES users ON UPDATE RESTRICT ON DELETE RESTRICT,
    alias      VARCHAR(100) NOT NULL,
    path       VARCHAR(255) NOT NULL,
    ordinal    INTEGER      NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE table servers
(
    id         UUID PRIMARY KEY,
    user_id    BIGINT       NULL NULL REFERENCES users ON UPDATE RESTRICT ON DELETE RESTRICT,
    url        VARCHAR(255) NOT NULL,
    username   VARCHAR(255),
    password   VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE table magnets
(
    id         UUID PRIMARY KEY,
    user_id    BIGINT       NULL NULL REFERENCES users ON UPDATE RESTRICT ON DELETE RESTRICT,
    url        VARCHAR(4096) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE table tasks
(
    id          UUID PRIMARY KEY,
    user_id     BIGINT      NOT NULL REFERENCES users ON UPDATE RESTRICT ON DELETE RESTRICT,
    server_id   UUID        NOT NULL REFERENCES servers ON UPDATE RESTRICT ON DELETE CASCADE,
    magnet_id   UUID        NOT NULL REFERENCES magnets ON UPDATE RESTRICT ON DELETE RESTRICT,
    status      task_status NOT NULL,
    description TEXT,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);