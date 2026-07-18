# ---- build stage: compile Rust -> WASM + static bundle ----
FROM node:22-slim AS build
WORKDIR /app
# Rust toolchain + wasm target (wasm32 links with rust's built-in lld, no C toolchain needed).
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.88.0 --target wasm32-unknown-unknown --profile minimal

COPY package.json package-lock.json ./
RUN npm ci                 # also materialises the fui-rs path dep under node_modules/
COPY . .
RUN npm run build          # emits the self-contained static site into public/

# ---- serve stage: plain static hosting ----
FROM nginx:1.27-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/public /usr/share/nginx/html
EXPOSE 80
