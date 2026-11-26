# The whole app in one image: the Rust server, serving the built web client.
#
# Images are pinned by digest, and each digest (not just its version) was
# published at least 14 days ago, as CLAUDE.md requires:
#   node:22.23.4-bookworm-slim   digest from 2026-11-05 (CI uses 22.23.5; this
#                                stage only builds static files)
#   rust:1.97.1-slim-bookworm    digest from 2026-08-11 (rust-toolchain.toml)
#   debian:bookworm-20261104-slim  dated release, digest from 2026-11-05
# Build:  docker build -t ricochet-robots .
# Run:            docker run -p 8080:8080 -v rr-data:/data ricochet-robots

FROM node:22.23.4-bookworm-slim@sha256:5ec23b7c622b00547753bc6ffd78085dd868a68aeb41c8e0a1418f033e13d96d AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
# No dependency needs an install script (as in CI).
RUN npm ci --ignore-scripts
COPY web/ ./
RUN npm run build

FROM rust:1.97.1-slim-bookworm@sha256:2775a09d208ff0d7c1f50490c45b62db929e87ba1dcbc3f2132ac71a704bcdd3 AS server
WORKDIR /src
# Use the image's toolchain without rustup re-checking components online.
# Keep this, the image tag and rust-toolchain.toml on one version.
ENV RUSTUP_TOOLCHAIN=1.97.1
COPY . .
RUN cargo build --release --locked -p server

FROM debian:bookworm-20261104-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
RUN useradd --system --uid 999 --home-dir /app --shell /usr/sbin/nologin rr \
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
