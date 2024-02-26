FROM ubuntu:latest
LABEL authors="m00n"

FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER=flashmind
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /flashmind
COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /flashmind
COPY --from=builder /flashmind/target/x86_64-unknown-linux-musl/release/flashmind-server ./

USER flashmind:flashmind

EXPOSE 3000
CMD ["/flashmind/flashmind-server"]