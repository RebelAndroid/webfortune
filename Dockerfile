FROM rustlang/rust:nightly

WORKDIR /app

COPY src src
COPY Cargo.lock Cargo.toml configuration.ini fortunes.csv ./

RUN cargo build --release

ENTRYPOINT ["/app/target/release/webfortune"]