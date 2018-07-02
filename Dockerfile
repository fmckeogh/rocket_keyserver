FROM ubuntu:bionic as build

#RUN apt-get update
#RUN apt install -y gnupg
#RUN apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys B97B0AFCAA1A47F044F244A07FCC7D46ACCC4CF8
#RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ precise-pgdg main" > /etc/apt/sources.list.d/pgdg.list

RUN apt update
RUN apt-get install -y git clang make pkg-config nettle-dev libssl-dev capnproto libsqlite3-dev curl libpq-dev software-properties-common musl-tools
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN /root/.cargo/bin/rustup default nightly
RUN /root/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

#RUN USER=root /root/.cargo/bin/cargo new --bin rocket_keyserver
WORKDIR /rocket_keyserver

#COPY ./Cargo.lock ./Cargo.lock
#COPY ./Cargo.toml ./Cargo.toml

#RUN /root/.cargo/bin/cargo build --release
#RUN rm -r src

COPY ./ ./
RUN /root/.cargo/bin/cargo build --release


FROM ubuntu:bionic

COPY --from=build /rocket_keyserver/Cargo.toml /
COPY --from=build /rocket_keyserver/Rocket.toml /
COPY --from=build /rocket_keyserver/target/release/rocket_keyserver /rocket_keyserver
COPY --from=build /root/.cargo/bin/diesel /diesel

ADD https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh wait-for-it.sh
RUN chmod +x wait-for-it.sh

RUN apt-get update
RUN apt-get install -y nettle-dev libpq-dev

EXPOSE 80
EXPOSE 443

ENV ROCKET_ENV prod
ENV DATABASE_URL postgres://postgres:password@db/keys
CMD ["sh", "-c", "./wait-for-it.sh db:5432 -- /diesel setup && /rocket_keyserver"]