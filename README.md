# uPaste Server

> Simple standalone pastebin

See "useful shell scripts" below for aliases to copy/paste from the command line.


## Development

* Clone this repo
* `cargo run -- admin database migrate`
* `cargo run -- serve`

## Running

* Build a docker image: `./docker.sh build`
    * env: `REGISTRY` to change the docker registry name used
* Run the image directly or
* Run using `./docker.sh run`:
    * env: `PORT_MAP` to change the container port mapping
    * Note: The script will pass the `--env-file .env.docker` to inject environment variables into the container
    
## Useful shell scripts

* `curl` and `jq` required

### In your *.bashrc*
```bash
# paste vars
export UPASTE_PASTEROOT=https://doma.in/new
export UPASTE_READROOT=https://doma.in
export UPASTE_READROOT_RAW=https://doma.in/raw

# Set to a value shared amongst your machines if
# you want to "securely" copy/paste between them.
# Content is de/encrypted server-side using the key
# provided. The "security" is encryption at rest and
# preventing others from stumbling on your paste's "code".
export UPASTE_ENCRYPTION_KEY=

## paste helpers
# copy from stdin or a file
# ex.
#  * copy from stdin: pc
#  * copy from file:  pc infile.txt
alias pc='~/bin/pc.sh'

# paste from a url or code
# ex.
#  * paste to stdout: pp $code
alias pp='~/bin/pp.sh'
```

*pc.sh*
```bash
#!/bin/bash
set -e

if [[ -z "$UPASTE_TTL_SECONDS" ]]; then
    ttl=
else
    ttl="ttl_seconds=$UPASTE_TTL_SECONDS"
fi

function post() {
    infile="@-"
    if [[ ! -z "$1" ]]; then
        infile="@$1"
    fi

    if [[ -z "$UPASTE_ENCRYPTION_KEY" ]]; then
        curl $UPASTE_PASTEROOT?$ttl -s --data-binary $infile
    else
        curl $UPASTE_PASTEROOT?$ttl -s --data-binary $infile -H "x-upaste-encryption-key: $UPASTE_ENCRYPTION_KEY"
    fi
}

if [ -z "$1" ]; then
    post | jq -r .key | echo "$UPASTE_READROOT_RAW/$(cat -)"
else
    post $1 | jq -r .key | echo "$UPASTE_READROOT_RAW/$(cat -)"
f
```

*pp.sh*
```bash
#!/bin/bash
set -e
if [[ -z "$1" ]]; then
    $0 "$(head -n 1)"
else
    if [[ $1 =~ ^http.* ]]; then
        if [[ -z "$UPASTE_ENCRYPTION_KEY" ]]; then
            curl $1 -s
        else
            curl $1 -s -H "x-upaste-encryption-key: $UPASTE_ENCRYPTION_KEY"
        fi
    else
        $0 $UPASTE_READROOT_RAW/$1
    fi
fi
```
