# Stage 1: Extract and Setup from official Postgres
FROM postgres:15-bookworm AS builder

# Set up environment for initdb
ENV PGDATA=/var/lib/postgresql/data
ENV POSTGRES_PASSWORD=dummy_pass_for_init

# Create necessary directories and initialize the database
RUN mkdir -p /var/lib/postgresql/data /var/run/postgresql && \
    chown -R postgres:postgres /var/lib/postgresql /var/run/postgresql && \
    # Create a temporary password file for initdb
    echo "1VMU2RKXjUP+2HBG/wBmdFHcOvdTg4Zi" > /tmp/pgpass && \
    # Run initdb as the postgres user with the password
    su postgres -c "/usr/lib/postgresql/15/bin/initdb -D /var/lib/postgresql/data --no-locale --encoding=UTF8 --pwfile=/tmp/pgpass" && \
    rm /tmp/pgpass && \
    # Start temporary postgres to create user and db
    su postgres -c "/usr/lib/postgresql/15/bin/pg_ctl -D /var/lib/postgresql/data -o '-c listen_addresses=\"\"' start" && \
    su postgres -c "psql -c \"CREATE USER kyx_admin WITH PASSWORD '1VMU2RKXjUP+2HBG/wBmdFHcOvdTg4Zi';\"" && \
    su postgres -c "psql -c \"CREATE DATABASE kyx_db OWNER kyx_admin;\"" && \
    su postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE kyx_db TO kyx_admin;\"" && \
    su postgres -c "/usr/lib/postgresql/15/bin/pg_ctl -D /var/lib/postgresql/data stop" && \
    # Scrub locales from postgresql.conf as Distroless doesn't have them
    sed -i '/lc_/d' /var/lib/postgresql/data/postgresql.conf && \
    sed -i '/en_US.utf8/d' /var/lib/postgresql/data/postgresql.conf && \
    # Enable SSL in postgresql.conf
    echo "ssl = on" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "ssl_cert_file = '/var/lib/postgresql/server.crt'" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "ssl_key_file = '/var/lib/postgresql/server.key'" >> /var/lib/postgresql/data/postgresql.conf && \
    # Allow SSL connections from all hosts in pg_hba.conf
    echo "hostssl all all all scram-sha-256" >> /var/lib/postgresql/data/pg_hba.conf

# Copy and secure certificates in builder
COPY infra/certs/cert.pem /tmp/server.crt
COPY infra/certs/key.pem /tmp/server.key
RUN chown 999:999 /tmp/server.crt /tmp/server.key && \
    chmod 644 /tmp/server.crt && \
    chmod 600 /tmp/server.key

RUN ln -s /usr/lib/postgresql/15/bin/postgres /usr/local/bin/postgres

# Stage 2: Distroless Final Stage
FROM gcr.io/distroless/cc-debian12

# Copy user/group definitions
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy Postgres binaries and libs
COPY --from=builder /usr/lib/postgresql /usr/lib/postgresql
COPY --from=builder /usr/local/bin /usr/local/bin
COPY --from=builder /usr/share/postgresql /usr/share/postgresql
COPY --from=builder /var/lib/postgresql /var/lib/postgresql
COPY --from=builder --chown=999:999 /var/run/postgresql /var/run/postgresql

# Copy system libraries (Postgres dependencies)
COPY --from=builder /lib/aarch64-linux-gnu /lib/aarch64-linux-gnu
COPY --from=builder /usr/lib/aarch64-linux-gnu /usr/lib/aarch64-linux-gnu

# Copy Certificates for SSL (preserved permissions from builder)
COPY --from=builder /tmp/server.crt /var/lib/postgresql/server.crt
COPY --from=builder /tmp/server.key /var/lib/postgresql/server.key

ENV PGDATA=/var/lib/postgresql/data
ENV LANG=C
USER 999:999

EXPOSE 5432

ENTRYPOINT ["/usr/lib/postgresql/15/bin/postgres"]
CMD [ \
    "-D", "/var/lib/postgresql/data", \
    "-c", "ssl=on", \
    "-c", "ssl_cert_file=/var/lib/postgresql/server.crt", \
    "-c", "ssl_key_file=/var/lib/postgresql/server.key", \
    "-c", "listen_addresses=*", \
    "-c", "lc_messages=C", \
    "-c", "lc_monetary=C", \
    "-c", "lc_numeric=C", \
    "-c", "lc_time=C" \
    ]
