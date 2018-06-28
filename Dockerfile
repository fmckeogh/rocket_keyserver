FROM ubuntu:18.04

COPY . .

RUN apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8
RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ precise-pgdg main" > /etc/apt/sources.list.d/pgdg.list

RUN apt-get update
RUN apt-get install -y git clang make pkg-config nettle-dev libssl-dev capnproto libsqlite3-dev curl libpq-dev python-software-properties software-properties-common postgresql-9.3 postgresql-client-9.3 postgresql-contrib-9.3
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH
RUN rustup default nightly
RUN cargo build --release

CMD ["./target/release/rocket_keyserver"]