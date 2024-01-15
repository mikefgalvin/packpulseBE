FROM rust:1.73.0

# WORKDIR /Users/mike/code/packpulseBE
COPY . .s

RUN cargo install --path  .

CMD ["packpulseBE"]