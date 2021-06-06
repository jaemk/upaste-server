# uPaste Server [![Build Status](https://travis-ci.org/jaemk/upaste-server.svg?branch=master)](https://travis-ci.org/jaemk/upaste-server)

> Simple standalone pastebin

Also see the general pasting client, [`upaste`](https://github.com/jaemk/upaste)


## Setup

* Clone this repo
* `cargo build --release`
* Update project files and download the latest binary release: `./update.py run`
* Create database and apply migrations:
    * `cargo run -- admin database migrate`
* Poke around: `cargo run -- admin database shell` (requires `sqlite3` to be installed)

## Running

* Run directly:
    * `target/release/upaste serve --port 8000 --public`
* With `systemd`:
    * Copy `upaste.service.sample` to `/lib/systemd/system/upaste.service` and update it with your user/proj-dir info.
    * Enable service: `sudo systemctl enable upaste.service` (see `upaste.service.sample` comments for more info).
    * `sudo systemctl start upaste`
* Behind a proxy (nginx):
    * Copy `nginx.conf.sample` to `/etc/nginx/sites-available`, update with your project info
    * `sudo nginx -t` to check config
    * Setup https certs, see `letsencrypt.info`
    * `sudo systemctl restart nginx`
* Clean out stale posts `target/release/upaste admin clean-before --days 30 --no-confirm` (haven't been viewed in 30 days)
    * An internal database sweeper will run every 10 minutes and delete pastes that haven't been
      viewed within the past 30 dats.

## Development

* Install [`rust`](https://rustup.rs/)
* Install [`migrant`](https://github.com/jaemk/migrant): db migration management
* `cargo run -- serve`

## Useful shell scripts

*.bashrc*
```bash
# paste vars
export UPASTE_PASTEROOT=https://doma.in/new
export UPASTE_READROOT=https://doma.in
export UPASTE_READROOT_RAW=https://doma.in/raw

## paste helpers
# copy from stdin or a file
alias pc='~/bin/pc.sh'
# paste from a url or code
alias pp='~/bin/pp.sh'
```

*pc.sh*
```bash
#!/bin/bash
set -e
if [ -z "$1" ]; then
    curl $UPASTE_PASTEROOT -s -d @- | jq -r .key | echo "$UPASTE_READROOT_RAW/$(cat -)"
else
    curl $UPASTE_PASTEROOT -s -d @$1 | jq -r .key | echo "$UPASTE_READROOT_RAW/$(cat -)"
fi
```

*pp.sh*
```bash
#!/bin/bash
set -e
if [[ -z "$1" ]]; then
    $0 "$(head -n 1)"
else
    if [[ $1 =~ ^http.* ]]; then
        curl $1 -s
    else
        $0 $UPASTE_READROOT_RAW/$1
    fi
fi
```