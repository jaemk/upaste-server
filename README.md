# uPaste Server [![Build Status](https://travis-ci.org/jaemk/upaste-server.svg?branch=master)](https://travis-ci.org/jaemk/upaste-server)

> Simple standalone pastebin clone

Also see the general pasting client, [`upaste`](https://github.com/jaemk/upaste)


## Setup

* Clone this repo
* Update project files and download the latest binary release: `./update.py run`
* Apply database migrations:
    * `bin/upaste admin database migrate`
    * Initial run will require setting up a `migrant` config with db-credentials
* Poke around: `bin/upaste admin database shell` (requires `sqlite3` to be installed)

## Running

* Run directly:
    * `bin/upaste serve --port 8000 --public`
* With `systemd`:
    * Copy `upaste.service.sample` to `/lib/systemd/system/upaste.service` and update it with your user/proj-dir info.
    * Enable service: `sudo systemctl enable upaste.service` (see `upaste.service.sample` comments for more info).
    * `sudo systemctl start upaste`
* Behind a proxy (nginx):
    * Copy `nginx.conf.sample` to `/etc/nginx/sites-available`, update with your project info
    * `sudo nginx -t` to check config
    * Setup https certs, see `letsencrypt.info`
    * `sudo systemctl restart nginx`
* Clean out stale posts `bin/upaste admin clean-before --days 30 --no-confirm` (haven't been viewed in 30 days)
    * An internal database sweeper will run every 10 minutes and delete pastes that haven't been
      viewed within the past 30 dats.

## Development

* Install [`rust`](https://rustup.rs/)
* Install [`migrant`](https://github.com/jaemk/migrant) (db migration manager): `cargo install migrant`
* `cargo run -- serve`

