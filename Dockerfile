FROM rust:latest

WORKDIR /usr/src/id-bars

COPY . .

RUN cargo build --release

RUN cargo install --path .

CMD ["/usr/local/cargo/bin/id-bars"]

