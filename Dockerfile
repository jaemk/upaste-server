# build the backend
FROM rust:1.52 as builder

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

# package
FROM alpine:3.14
RUN mkdir /upaste
WORKDIR /upaste

RUN mkdir ./bin
COPY --from=builder /upaste/target/release/upaste ./bin/upaste
COPY --from=builder /upaste/commit_hash.txt ./commit_hash.txt

# copy all static files
COPY ./migrations ./migrations
COPY ./templates ./templates
COPY ./assets ./assets

# set the startup command to run your binary
CMD ["./bin/upaste", "serve", "--port", "80", "--public"]
