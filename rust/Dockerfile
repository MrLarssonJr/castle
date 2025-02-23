ARG RUST_VERSION=1.76
ARG CARGO_CHEF_VERSION=0.1.65
ARG DEBIAN_VERSION=bookworm
FROM lukemathwalker/cargo-chef:${CARGO_CHEF_VERSION}-rust-${RUST_VERSION}-${DEBIAN_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP_NAME=unset
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin $APP_NAME

ARG DEBIAN_VERSION=bookworm
FROM debian:${DEBIAN_VERSION} AS final
ARG APP_NAME=unset
WORKDIR /app
# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser
COPY --from=builder /app/target/release/$APP_NAME /bin/server

# What the container should run when it is started.
CMD ["/bin/server"]
