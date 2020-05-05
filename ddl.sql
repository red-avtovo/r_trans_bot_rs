CREATE TYPE task_status AS ENUM ('created', 'started', 'finished', 'error');

CREATE table users (
    id INTEGER PRIMARY KEY,
    chat INTEGER UNIQUE NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    username VARCHAR(255)
);

CREATE table dirs (
    id UUID PRIMARY KEY,
    user_id INTEGER NOT NULL,
    alias VARCHAR(100) NOT NULL,
    path VARCHAR(255) NOT NULL,
    ordinal INTEGER NOT NULL

    CONSTRAINT dirs_user_id_users_id_fkey FOREIGN KEY (user_id)
      REFERENCES users (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION
);

CREATE table tasks (
    id UUID PRIMARY KEY,
    user_id INTEGER NOT NULL,
    magnet VARCHAR(100) NOT NULL,
    directory UUID NOT NULL,
    status task_status NOT NULL,
    description TEXT,

    CONSTRAINT tasks_user_id_users_id_fkey FOREIGN KEY (user_id)
      REFERENCES users (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT tasks_directory_dirs_id_fkey FOREIGN KEY (directory)
      REFERENCES dirs (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION
);