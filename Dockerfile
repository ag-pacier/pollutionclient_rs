FROM rust:slim-bookworm

WORKDIR /usr/src/pollutionclient_rs
COPY . .

RUN cargo install --path .

CMD ["pollutionclient_rs"]