# Remote Transmission Telegram Bot

![CI](https://github.com/red-avtovo/r_trans_bot_rs/workflows/CI/badge.svg)

This bot allows you to send magnet links to bot and start tasks to preconfigured server with preconfigured directories

## Build locally

```bash
docker build -t bot .
```

### Run with local .env file

```bash
docker run --rm -it --env-file ./.env bot
```

### Migration with diesel cli

    cargo install diesel_cli

#### Init test database

    diesel --database-url="postgres://postgres:mysecretpassword@172.17.0.4:5432/postgres" migration run


| [![](https://www.iconfinder.com/icons/986956/download/png/24) Use the bot](https://t.me/RTransBot) |
| ---- |

Support the project: [![Donate button](https://www.paypalobjects.com/en_US/DK/i/btn/btn_donateCC_LG.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=H337RKJSC4YG4&source=url)
