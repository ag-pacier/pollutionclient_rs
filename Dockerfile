FROM rust:slim-bookworm

WORKDIR /usr/src/pollutionclient_rs
COPY . .

RUN cargo install --path .
RUN adduser rustuser
USER rustuser

CMD ["pollutionclient_rs"]