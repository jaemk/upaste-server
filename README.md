# uPaste Server [![Build Status](https://travis-ci.org/jaemk/upaste-server.svg?branch=master)](https://travis-ci.org/jaemk/upaste-server)

> Basic hastebin clone

Also see the general pasting client, [`upaste`](https://github.com/jaemk/upaste)


## Setup

* create your postgres database and user, or use the helper script: `./create_db.sh --help`
* install dependencies: `libpq-dev`
* download the latest binary release: `./update.py`
* `./bin/upaste admin migrate`
    * initial run will require setting up a `migrant` config with db-credentials
* poke around: `./target/release/upaste admin shell`

## Running

* run directly:
    * `./bin/upaste serve --port 8000 --public --log`
* with `systemd`:
    * copy `upaste.service.sample` to `/etc/systemd/system/upaste.service` and update it with your user/proj-dir info. Enable service, `sudo systemctl enable upaste.service` (see `upaste.service.sample` comments for more info).
    * `sudo systemctl start upaste`
* behind a proxy (nginx):
    * copy `nginx.conf.sample` to `/etc/nginx/sites-available`, update with your project info, `sudo nginx -t` to check config
    * setup https certs, see `letsencrypt.info`
    * `sudo systemctl restart nginx`
* clean out stale posts `./target/release/upaste admin clean-before --days 30 --no-confirm`

## Development

* install [`rust`](https://rustup.rs/)
* install [`migrant`](https://github.com/jaemk/migrant) (db migration manager): `cargo install migrant`
* `cargo build`
