FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY release-artifacts/linux-${TARGETARCH}/flowl /usr/local/bin/flowl
RUN chmod +x /usr/local/bin/flowl

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:${FLOWL_PORT:-8080}/health || exit 1

USER 1000:1000

CMD ["/usr/local/bin/flowl"]
