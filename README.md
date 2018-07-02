# rocket_keyserver
> PGP keyserver using Rocket

[![Build Status](https://travis-ci.org/chocol4te/rocket_keyserver.svg?branch=master)](https://travis-ci.org/chocol4te/rocket_keyserver) [![size/layers](https://images.microbadger.com/badges/image/chocol4te/rocket_keyserver.svg)](https://microbadger.com/images/chocol4te/rocket_keyserver)

## Usage

`docker-compose up` pulls the latest image and deploys alongside a PostgreSQL container, running on ports 80 and 443.

`cargo run` compiles and runs in a development configuration on https://localhost:8000. `DATABASE_URL` must be set to a valid PostgreSQL instance.

## Todo

- [x] Implement multistage Docker builds for reasonable image size
- [x] Better error handling
- [ ] Write better internal tests, current ones quite poor
- [ ] Write `docker-compose` black box tests
- [ ] Write benchmarks to ensure no performance regressions occur

## Contributing

Issues and PRs very welcome, nothing is too small.
PRs must pass all tests, have run `cargo fmt` and `cargo clippy`.

## License
GNU Affero General Public License v3.0([LICENSE](LICENSE) or
  https://www.gnu.org/licenses/agpl-3.0.txt)
