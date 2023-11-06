# Remote Transmission Telegram Bot

[![](https://img.shields.io/badge/Powered%20by-Diesel-red)](https://diesel.rs/guides/getting-started) ![CI](https://github.com/red-avtovo/r_trans_bot_rs/workflows/CI/badge.svg)



This bot allows you to send magnet links to bot and start tasks to preconfigured server with preconfigured directories

## Build locally

```bash
docker build -t bot .
```

or

```
# install rustup
sudo apt install gcc pkg-config libssl-dev
cargo build
```

### Run with local .env file

```bash
docker run --rm -it --env-file ./.env bot
```

### Migration with diesel cli

    cargo install diesel_cli --no-default-features --features postgres

### Prepare for testing
  
    docker compose up -d && sleep 3 && export $(grep -v '^#' .env.test | xargs -0) && diesel migration run

#### Optionally remote init test database
    
    diesel --database-url="postgres://postgres:mysecretpassword@172.17.0.4:5432/postgres" migration run

Key features:

- Download torrents by:
  - magnet link
  - rutracker page link
  - sent file torrent (*)
- Support multiple servers
- Friend another user (with friend code)
- Share server with a friend
- Share directories with a friend (with poll)
- Download to a friends server
- Manage sharing
- Rename directory aliases
- Perform actions with confirmation
- Persist resolved transmission task name even after deletion

| [![](https://www.iconfinder.com/icons/986956/download/png/24) Use the bot](https://t.me/RTransBot) |
|----------------------------------------------------------------------------------------------------|

Support the project: [![Donate button](https://www.paypalobjects.com/en_US/DK/i/btn/btn_donateCC_LG.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=H337RKJSC4YG4&source=url)
