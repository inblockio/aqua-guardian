FROM rust:1.77.2-bullseye as build

# protobuf compiler
RUN apt update && apt install -y --no-install-recommends \
protobuf-compiler \
libprotobuf-dev \
&& rm -rf /var/lib/apt/lists/*

COPY . /tmp/aqua-guardian
WORKDIR /tmp/aqua-guardian
RUN cargo build --release -p guardian
RUN cargo test --workspace --exclude node-eth-lookup

FROM scratch
COPY --from=build /tmp/aqua-guardian/target/release/guardian /bin/guardian
CMD ["/bin/guardian"]
