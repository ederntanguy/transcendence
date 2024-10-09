FROM rust:1.76-bookworm as builder
RUN apt update && apt install -y cmake
WORKDIR /usr/src/pong-serv
COPY Cargo.toml ./
COPY src/ ./src/
RUN cargo install --path .

FROM ubuntu:mantic
RUN apt update; \
    apt upgrade -y; \
    apt install -y netcat-openbsd libssl3 ca-certificates; \
    rm -rf /var/lib/apt/lists/*;
COPY --from=builder /usr/local/cargo/bin/pong-serv /usr/local/bin/pong-serv
COPY health_check /
COPY --chown=root:root --chmod=0644 tls/transcendence.der.crt /tls/
COPY --chown=root:root --chmod=0400 tls/transcendence.der.key /tls/
COPY --chown=root:root --chmod=0644 tls/root_ca.pem.crt /usr/local/share/ca-certificates/
RUN update-ca-certificates
EXPOSE 8081
CMD ["pong-serv", "-p", "8081", "-l", "/var/log/pong", "-c", "err", "/tls/transcendence.der.key", "/tls/transcendence.der.crt"]
