# uPaste Server [![Build Status](https://travis-ci.org/jaemk/upaste-server.svg?branch=master)](https://travis-ci.org/jaemk/upaste-server)

> Small uPaste server inspired by hastebin

> wip

# Setup

* install [rust](https://rustup.rs/)
* install `diesel_cli`: `cargo install diesel_cli --no-default-features --features postgres`
* create your postgres database and user
* copy `.env.sample` to `.env` and update to your database url/user/pass
* build a release artifact, `cargo build --release`
* copy `upaste.conf.sample` to `/etc/init/upaste.conf` and update to your user/proj-dir, `sudo initctl reload-configuration` (see comments for more info)
* copy `nginx.conf.sample` to `/etc/nginx/sites-available`, update project info, `sudo nginx -t` to check config
* setup https certs, see `letsencrypt.info`
* run! `sudo service upaste start` and `sudo service nginx restart`
