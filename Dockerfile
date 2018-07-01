FROM alpine:edge as build

#RUN apt-get update
#RUN apt install -y gnupg
#RUN apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8
#RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ precise-pgdg main" > /etc/apt/sources.list.d/pgdg.list

RUN apk update
RUN apt-get install -y git clang make pkg-config nettle-dev libssl-dev capnproto libsqlite3-dev curl libpq-dev software-properties-common musl-tools
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN /root/.cargo/bin/rustup default nightly
RUN /root/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

ENV DATABASE_URL postgres://postgres:password@db/keys
ENV PKG_CONFIG_ALLOW_CROSS 1

RUN USER=root /root/.cargo/bin/cargo new --bin rocket_keyserver
WORKDIR /rocket_keyserver

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN /root/.cargo/bin/cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN /root/.cargo/bin/cargo build --release --target=x86_64-unknown-linux-musl -C linker=musl-gcc


FROM alpine:edge

COPY --from=build /rocket_keyserver/target/x86_64-unknown-linux-musl/release/rocket_keyserver /rocket_keyserver

ADD https://raw.githubusercontent.com/eficode/wait-for/master/wait-for wait-for
RUN chmod +x wait-for

RUN apk add nettle-dev gmp libpq

EXPOSE 80
EXPOSE 443

ENV ROCKET_ENV prod
CMD ["sh", "-c", "./wait-for db:5432; /root/.cargo/bin/diesel setup; /rocket_keyserver"]