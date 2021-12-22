FROM rust:1.56 as builder
COPY . .
RUN cargo build --releas
EXPOSE 8080
CMD ["./target/debug/poggers"]
