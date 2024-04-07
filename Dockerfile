FROM rust:1.77
RUN apt-get update && apt-get install -y python3 lua5.3 && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/dyno_code"]
