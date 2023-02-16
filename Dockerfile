# build server
FROM rustlang/rust:nightly as server-builder
WORKDIR /usr/src/server
COPY ./server .
RUN cargo install --path .

# build client
FROM node:18-alpine AS client-builder

COPY client/package.json client/package.json
COPY client/yarn.lock client/yarn.lock
RUN cd client && yarn install --frozen-lockfile

COPY client/src client/src
COPY client/public client/public
COPY client/tsconfig.json client/tsconfig.json

RUN cd client && yarn run build

FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=client-builder /src/shared server/src/shared
COPY --from=server-builder /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["server"]