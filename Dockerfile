FROM rust:1.56 as builder

RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --bin witness
WORKDIR /witness

ADD . ./

RUN cargo build --target x86_64-unknown-linux-musl --release 

FROM alpine:3.14
ARG APP=/usr/src/app

EXPOSE 3030

COPY --from=builder /witness/target/x86_64-unknown-linux-musl/release/keri-witness-http ${APP}/witness 

USER $APP_USER
WORKDIR ${APP}

CMD ["./witness"]