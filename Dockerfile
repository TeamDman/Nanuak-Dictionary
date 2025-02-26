# https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

FROM rust:1.85 as build

# create a new empty shell project
RUN USER=root cargo new --bin nanuak_dictionary
WORKDIR /nanuak_dictionary

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/nanuak_dictionary*
RUN cargo build --release

# our final base
FROM rust:1.85-slim-bookworm

# copy the build artifact from the build stage
COPY --from=build /nanuak_dictionary/target/release/nanuak_dictionary .

# set the startup command to run your binary
CMD ["./nanuak_dictionary"]