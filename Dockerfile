FROM debian:buster

COPY . .

#RUN apt-get update
#RUN apt install -y gnupg
#RUN apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8
#RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ precise-pgdg main" > /etc/apt/sources.list.d/pgdg.list

RUN apt-get update
RUN apt-get install -y git clang make pkg-config nettle-dev libssl-dev capnproto libsqlite3-dev curl libpq-dev software-properties-common
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN /root/.cargo/bin/rustup default nightly

ENV DATABASE_URL postgres://postgres:password@db/keys

RUN /root/.cargo/bin/cargo build --release

RUN /root/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

ADD https://raw.githubusercontent.com/eficode/wait-for/master/wait-for wait-for
RUN chmod +x wait-for

EXPOSE 80
EXPOSE 443

ENV ROCKET_ENV prod
CMD ["sh", "-c", "./wait-for db:5432; /root/.cargo/bin/diesel setup; /target/release/rocket_keyserver"]