FROM debian:buster

COPY . .

#RUN apt-get update
#RUN apt install -y gnupg
#RUN apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8
#RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ precise-pgdg main" > /etc/apt/sources.list.d/pgdg.list

RUN apt-get update
RUN apt-get install -y git clang make pkg-config nettle-dev libssl-dev capnproto libsqlite3-dev curl libpq-dev software-properties-common postgresql-10 postgresql-client-10 postgresql-contrib-10
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH
RUN rustup default nightly

ENV DATABASE_URL postgres://postgres:password@localhost/keys

RUN echo "/etc/init.d/postgresql start && exit 0" > /etc/rc.local
RUN /etc/init.d/postgresql start &&\
    su postgres -c "psql --command \"ALTER USER postgres WITH PASSWORD 'password';\" "

RUN cargo build --release

RUN cargo install diesel_cli --no-default-features --features postgres

CMD ["/bin/bash -c "/etc/init.d/postgresql restart && diesel setup && /target/release/rocket_keyserver""]