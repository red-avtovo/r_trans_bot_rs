CREATE TYPE task_status AS ENUM ('created', 'started', 'finished', 'error');

CREATE table users (
    id INTEGER PRIMARY KEY,
    chat INTEGER UNIQUE NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    username VARCHAR(255),
    salt VARCHAR(255) Not NULL
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
    server_id UUID NOT NULL,
    magnet VARCHAR(100) NOT NULL,
    status task_status NOT NULL,
    description TEXT,

    CONSTRAINT tasks_user_id_users_id_fkey FOREIGN KEY (user_id)
      REFERENCES users (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION,
    
    CONSTRAINT tasks_server_id_servers_id_fkey FOREIGN KEY (server_id)
      REFERENCES servers (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION
);

CREATE table servers (
  id UUID PRIMARY KEY,
  user_id INTEGER NULL NULL,
  url VARCHAR(255) NOT NULL,
  username VARCHAR(255),
  password VARCHAR(255),

  CONSTRAINT servers_user_id_users_id_fkey FOREIGN KEY (user_id)
      REFERENCES users (id) MATCH SIMPLE
      ON UPDATE NO ACTION ON DELETE NO ACTION
)