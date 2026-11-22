FROM rust:1-bookworm AS base

RUN rustup component add clippy

WORKDIR /workspace
COPY Cargo.toml Cargo.lock LICENSE-MIT LICENSE-APACHE NOTICE ./
COPY src ./src
COPY tests ./tests

FROM base AS dev
CMD ["cargo", "test", "--lib", "--bins", "--tests"]

FROM base AS verify
RUN cargo clippy --all-targets -- -D warnings \
    && cargo test --lib --bins --verbose \
    && cargo test --test bench_speedcoding --test phase_1_core_types --test phase_2_parser --test phase_3_arithmetic_math --test phase_4_control_addressing --test phase_5_io_storage --test phase_6_checking_listing_timing --test e2e_speedcoding --verbose \
    && cargo build --release --verbose \
    && cargo test --release --lib --bins --tests --verbose \
    && ./target/release/speedcoding --version \
    && ./target/release/speedcoding info \
    && printf '0300 ADD 0 0500 0501 0502 - 0000\n0301 PRINT 0 0502 0502 0101 STOP 0301\n' > /tmp/smoke.sco \
    && ./target/release/speedcoding run /tmp/smoke.sco --data 500=12.5 --data 501=29.5 | grep -q 'ProgramStop' \
    && ./target/release/speedcoding trace /tmp/smoke.sco --data 500=12.5 --data 501=29.5 | grep -q 'Listing lines:' \
    && rm -f /tmp/smoke.sco \
    && cargo doc --no-deps \
    && cargo package --list --allow-dirty | tee /tmp/pkg.list \
    && ! grep -q '102632810-05-01-acc.pdf' /tmp/pkg.list \
    && rm -f /tmp/pkg.list

FROM base AS builder
RUN cargo build --release --verbose

FROM debian:bookworm-slim AS runtime
COPY --from=builder /workspace/target/release/speedcoding /usr/local/bin/speedcoding
COPY LICENSE-MIT LICENSE-APACHE NOTICE /usr/share/doc/speedcoding/
ENTRYPOINT ["speedcoding"]
CMD ["--help"]
