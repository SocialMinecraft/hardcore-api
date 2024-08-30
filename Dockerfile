FROM rust:1.80

WORKDIR /app
COPY . .

RUN cargo install --path .

EXPOSE 3030
CMD ["HardcoreApi"]