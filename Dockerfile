#FROM node:16.11-bullseye-slim as frontend
#RUN apt-get update && apt-get install -y make
#COPY package.json yarn.lock /src/
#WORKDIR /src
#RUN yarn install
#COPY . .
#RUN make frontend

FROM rust:slim as rust
WORKDIR /src
RUN apt-get update && apt-get install -y git pkg-config libssl-dev
#RUN mkdir -p \ 
#	/src/database/db-core/src \
#	/src/database/db-sqlx-postgres/src \
#	/src/database/db-sqlx-sqlite/src \
#	/src/database/migrator/src \
#	/src/src \
#	/src/assets \
#	/src/static
#RUN touch \
#	/src/src/main.rs \
#	/src/database/db-core/src/lib.rs \
#	/src/database/db-sqlx-postgres/src/lib.rs \
#	/src/database/db-sqlx-sqlite/src/lib.rs \
#	/src/database/migrator/src/main.rs \
#	/src/database/migrator/src/main.rs
#COPY Cargo.toml Cargo.lock /src/
#COPY ./database/db-core/Cargo.toml /src/database/db-core/
#COPY ./database/db-sqlx-postgres/Cargo.toml /src/database/db-sqlx-postgres/
#COPY ./database/db-sqlx-postgres/migrations/ /src/database/db-sqlx-postgres/
#COPY ./database/db-sqlx-sqlite/Cargo.toml /src/database/db-sqlx-sqlite/
#COPY ./database/db-sqlx-sqlite/migrations/ /src/database/db-sqlx-sqlite/
#COPY ./database/migrator/Cargo.toml /src/database/migrator/
#RUN cargo build --release || true
#COPY database/ /src/
COPY . /src/
RUN cargo build --release

FROM debian:bullseye-slim
RUN useradd -ms /bin/bash -u 1001 gitpad
WORKDIR /home/gitpad
COPY --from=rust /src/target/release/gitpad /usr/local/bin/
COPY --from=rust /src/config/default.toml /etc/gitpad/config.toml
USER gitpad
LABEL org.opencontainers.image.source https://github.com/realaravinth/gitpad
LABEL org.opencontainers.image.authors "Aravinth Manivannan"
LABEL org.opencontainers.image.license "AGPL-3.0-or-later"
LABEL org.opencontainers.image.title "GitPad"
LABEL org.opencontainers.image.description "Self-hosted GitHub Gists"
CMD [ "/usr/local/bin/gitpad" ]
