FROM rust:1.77-slim-buster
WORKDIR /usr/src/dyno_code_api
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/dyno_code"]
