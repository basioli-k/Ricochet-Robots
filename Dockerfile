# The whole app in one image: the Rust server, serving the built web client.
#
# Images are pinned by digest. The build stages use the Node and Rust
# versions CI uses (both past the 14-day rule); only static files and the
# server binary leave them. The runtime is a dated Debian release, also past
# 14 days. Build:  docker build -t ricochet-robots .
# Run:            docker run -p 8080:8080 -v rr-data:/data ricochet-robots

FROM node:22.23.5-bookworm-slim@sha256:be979e5523c76fa4d5ca4ebd3099078f220aa47ec39c48ad5b5df9314debd6eb AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
# No dependency needs an install script (as in CI).
RUN npm ci --ignore-scripts
COPY web/ ./
RUN npm run build

FROM rust:1.97.1-slim-bookworm@sha256:2775a09d208ff0d7c1f50490c45b62db929e87ba1dcbc3f2132ac71a704bcdd3 AS server
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p server

FROM debian:bookworm-20261104-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
RUN useradd --system --uid 10001 --home-dir /app --shell /usr/sbin/nologin rr \
    && mkdir /data && chown rr /data
COPY --from=server /src/target/release/server /app/server
COPY --from=web /src/web/dist /app/web
# Behind HTTPS, also set RR_PUBLIC_BASE_URL (e.g. https://ricochet.example):
# it decides Secure cookies and the Origin the game socket accepts.
ENV RR_BIND=0.0.0.0:8080 \
    RR_WEB_DIR=/app/web \
    RR_DATABASE_URL=sqlite:///data/ricochet.db \
    RR_PUBLIC_BASE_URL=http://localhost:8080
VOLUME /data
USER rr
EXPOSE 8080
CMD ["/app/server"]
