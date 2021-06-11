# build the backend
FROM rust:1.52

# create a new empty shell
RUN USER=root cargo new --bin upaste
WORKDIR /upaste

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy all source files
COPY ./src ./src

# save git hash of this build
COPY ./.git ./.git
RUN git rev-parse HEAD | awk '{ printf "%s", $0 >"commit_hash.txt" }'
RUN rm -rf ./.git

# build for release
RUN rm ./target/release/deps/upaste*
RUN cargo build --release

# copy all static files
COPY ./migrations ./migrations
COPY ./templates ./templates
COPY ./assets ./assets

RUN mkdir ./bin
RUN cp ./target/release/upaste ./bin/upaste
RUN rm -rf ./target

# set the startup command to run your binary
CMD ["./bin/upaste", "serve", "--port", "80", "--public"]
