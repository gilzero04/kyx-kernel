FROM rust:1.92-slim-bookworm AS builder

# Build arguments for version metadata
ARG APP_VERSION=0.1.0
ARG BUILD_DATE=unknown
ARG GIT_COMMIT=unknown

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY .sqlx ./.sqlx
# Touch main.rs to ensure rebuild
RUN touch src/main.rs

# Build the actual application with offline metadata
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ============================================
# Runtime Stage (minimal image)
# ============================================
FROM debian:bookworm-slim

# Re-declare ARGs for this stage
ARG APP_VERSION=0.1.0
ARG BUILD_DATE=unknown
ARG GIT_COMMIT=unknown

# Set as ENV for runtime access
ENV APP_VERSION=${APP_VERSION}
ENV BUILD_DATE=${BUILD_DATE}
ENV GIT_COMMIT=${GIT_COMMIT}

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    netcat-openbsd \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/kyx-kernel /app/kyx-kernel



# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Non-root user for security
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./kyx-kernel"]
