FROM clux/muslrust as build

RUN cd / && git clone https://github.com/diesel-rs/diesel.git && cd diesel/diesel_cli && cargo build --release --target=x86_64-unknown-linux-musl --no-default-features --features postgres
RUN x86_64-linux-gnu-strip /diesel/target/x86_64-unknown-linux-musl/release/diesel
RUN USER=root cargo new --bin prebuild && mv prebuild/* . && rm -r prebuild
COPY ./Cargo.toml .
RUN cargo build --release

COPY . .
RUN cargo build --release
RUN x86_64-linux-gnu-strip /volume/target/x86_64-unknown-linux-musl/release/rocket_keyserver


FROM alpine:edge

EXPOSE 80
EXPOSE 443

COPY ./Cargo.toml /
COPY ./Rocket.toml /
COPY --from=build /volume/target/x86_64-unknown-linux-musl/release/rocket_keyserver /rocket_keyserver
COPY --from=build /diesel/target/x86_64-unknown-linux-musl/release/diesel /diesel

ADD https://github.com/eficode/wait-for/raw/master/wait-for /wait-for
RUN chmod +x /wait-for

ENV ROCKET_ENV prod
ENV DATABASE_URL postgres://postgres:password@db/keys

CMD ["sh", "-c", "/wait-for db:5432 -- /diesel setup && /rocket_keyserver"]