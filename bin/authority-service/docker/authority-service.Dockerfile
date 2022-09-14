##############################################
## Builder                                  ##
##############################################
FROM rust:latest AS builder

RUN rustup default nightly

RUN USER=root cargo new authority-service
WORKDIR /authority-service

COPY ./bin/authority-service ./

RUN cargo build

##############################################
## Final image                              ##
##############################################
FROM rust:latest

COPY --from=builder /authority-service/target/debug/authority-service .
COPY --from=builder /authority-service/Rocket.toml .

EXPOSE 8000

CMD ["./authority-service"]
