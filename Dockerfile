# The website in a container, for any Docker host (Render, Fly.io, Cloud Run…). The
# private git dependencies are read with a BuildKit secret, never stored in the image:
#
#   REPOS_TOKEN=… docker build --secret id=repos_token,env=REPOS_TOKEN -t duova .
#   docker run --rm -p 8080:8080 duova

FROM rust:1-slim-trixie AS build
RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY site ./site
WORKDIR /src/site
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN --mount=type=secret,id=repos_token \
    if [ -f /run/secrets/repos_token ]; then \
      export REPOS_TOKEN="$(cat /run/secrets/repos_token)" GIT_CONFIG_COUNT=1 \
        GIT_CONFIG_KEY_0=credential.https://github.com.helper \
        GIT_CONFIG_VALUE_0='!f() { test "$1" = get && echo username=x-access-token && echo "password=$REPOS_TOKEN"; }; f'; \
    fi \
    && cargo build --release --bin duova-site \
    && cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && cargo install wasm-bindgen-cli --version "$version" --locked --bin wasm-bindgen \
    && wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
        --out-dir pkg target/wasm32-unknown-unknown/wasm-release/duova_site.wasm \
    && cp target/release/duova-site /server

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 duova
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /src/site/public ./public
COPY --from=build /src/site/pkg ./pkg
USER duova
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
