# build server
FROM rustlang/rust:nightly as server-builder
WORKDIR /usr/src/server
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=server-builder /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["server"]