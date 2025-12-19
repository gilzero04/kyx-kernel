# Stage 1: Build/Extract from official Redis Stack
FROM redis/redis-stack-server:latest AS builder

# Stage 2: Distroless Final Stage
FROM gcr.io/distroless/cc-debian12

# Copy binaries
COPY --from=builder /usr/bin/redis-server /usr/bin/redis-server
COPY --from=builder /usr/bin/redis-cli /usr/bin/redis-cli

# Copy Redis Stack modules
COPY --from=builder /opt/redis-stack /opt/redis-stack

# Copy Certificates for TLS
COPY infra/certs/cert.pem /etc/redis/certs/redis.crt
COPY infra/certs/key.pem /etc/redis/certs/redis.key

EXPOSE 6379 6380

ENTRYPOINT ["/usr/bin/redis-server"]
CMD [ \
    "--tls-port", "6380", \
    "--port", "0", \
    "--tls-cert-file", "/etc/redis/certs/redis.crt", \
    "--tls-key-file", "/etc/redis/certs/redis.key", \
    "--tls-auth-clients", "no", \
    "--loadmodule", "/opt/redis-stack/lib/rejson.so", \
    "--loadmodule", "/opt/redis-stack/lib/redisearch.so" \
    ]
