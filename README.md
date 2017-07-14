# uPaste Server [![Build Status](https://travis-ci.org/jaemk/upaste-server.svg?branch=master)](https://travis-ci.org/jaemk/upaste-server)

> hastebin clone

Also see the general pasting client, [`upaste`](https://github.com/jaemk/upaste)


## Setup

* Create your postgres database and user, or use the helper script: `./create_db.sh --help`
* install dependencies: `libpq-dev`
* Update project files and download the latest binary release: `./update.py run`
* Apply database migrations:
    * `./bin/upaste admin database migrate`
    * Initial run will require setting up a `migrant` config with db-credentials
* Poke around: `./bin/upaste admin database shell`

## Running

* Run directly:
    * `./bin/upaste serve --port 8000 --public --log`
* With `systemd`:
    * Copy `upaste.service.sample` to `/lib/systemd/system/upaste.service` and update it with your user/proj-dir info.
    * Enable service: `sudo systemctl enable upaste.service` (see `upaste.service.sample` comments for more info).
    * `sudo systemctl start upaste`
* Behind a proxy (nginx):
    * Copy `nginx.conf.sample` to `/etc/nginx/sites-available`, update with your project info
    * `sudo nginx -t` to check config
    * Setup https certs, see `letsencrypt.info`
    * `sudo systemctl restart nginx`
* Clean out stale posts `./bin/upaste admin clean-before --days 30 --no-confirm`

## Development

* Install [`rust`](https://rustup.rs/)
* Install [`migrant`](https://github.com/jaemk/migrant) (db migration manager): `cargo install migrant`
* `cargo run -- serve --log`
