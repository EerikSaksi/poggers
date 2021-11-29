FROM rust as builder
COPY . .
RUN cargo build --release 
CMD ["./target/release/poggers"]
