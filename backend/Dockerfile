# Stage 1: Builder
FROM rust:1.88-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
# Cache dependencies layer
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM alpine:3.20

RUN apk add --no-cache ca-certificates ffmpeg

WORKDIR /app

COPY --from=builder /app/target/release/errant_fox ./errant_fox
COPY exchange_viewer_target.html ./exchange_viewer_target.html

EXPOSE 8080

CMD ["sh", "-c", "mkdir -p /data/db /data/previews /data/avatars && ./errant_fox"]
