FROM rust as builder
COPY . .
RUN cargo build 
EXPOSE 8080
CMD ["./target/debug/poggers"]
