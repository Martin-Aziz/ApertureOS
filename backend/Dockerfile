FROM rust:1.83-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
COPY backend/Cargo.toml backend/Cargo.lock* /app/backend/
RUN mkdir -p /app/backend/src && echo "fn main() {}" > /app/backend/src/main.rs
RUN cd /app/backend && cargo build --release && rm -rf src
COPY backend /app/backend
RUN cd /app/backend && cargo build --release

FROM alpine:3.21
RUN addgroup -S app && adduser -S app -G app
WORKDIR /srv
RUN apk add --no-cache ca-certificates wget
COPY --from=builder /app/backend/target/release/pixelforge-backend /srv/pixelforge-backend
USER app
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --retries=5 CMD wget --spider -q http://127.0.0.1:8080/health/live || exit 1
ENTRYPOINT ["/srv/pixelforge-backend"]
