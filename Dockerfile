
FROM rust:latest

WORKDIR /usr/src/app


COPY . .


RUN cargo run --release

# RUN ["./target/release/orders"] 
