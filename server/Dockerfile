FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/server /usr/local/bin/server
EXPOSE 8080
CMD ["server"]