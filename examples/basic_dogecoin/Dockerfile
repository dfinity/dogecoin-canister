# Always use the latest network launcher image
# Before building we must pull to pick up new releases
# For real world usage, consider pinning the version instead of using :latest
FROM ghcr.io/dfinity/icp-cli-network-launcher:latest

ARG TARGETARCH
ARG DOGECOIN_VERSION=1.14.9

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && \
    case "${TARGETARCH}" in \
      "amd64") \
        DOGECOIN_TARBALL="dogecoin-${DOGECOIN_VERSION}-x86_64-linux-gnu.tar.gz" ; \
        DOGECOIN_SHA256="4f227117b411a7c98622c970986e27bcfc3f547a72bef65e7d9e82989175d4f8" ;; \
      "arm64") \
        DOGECOIN_TARBALL="dogecoin-${DOGECOIN_VERSION}-aarch64-linux-gnu.tar.gz" ; \
        DOGECOIN_SHA256="6928c895a20d0bcb6d5c7dcec753d35c884a471aaf8ad4242a89a96acb4f2985" ;; \
      *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/dogecoin/dogecoin/releases/download/v${DOGECOIN_VERSION}/${DOGECOIN_TARBALL}" \
      -o /tmp/dogecoin.tar.gz && \
    echo "${DOGECOIN_SHA256}  /tmp/dogecoin.tar.gz" | sha256sum -c && \
    tar xzf /tmp/dogecoin.tar.gz --strip-components=2 \
      -C /usr/local/bin \
      "dogecoin-${DOGECOIN_VERSION}/bin/dogecoind" \
      "dogecoin-${DOGECOIN_VERSION}/bin/dogecoin-cli" && \
    rm /tmp/dogecoin.tar.gz && \
    apt-get purge -y curl && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*

COPY docker/start.sh /app/start.sh
RUN chmod +x /app/start.sh

ENTRYPOINT ["/app/start.sh"]
