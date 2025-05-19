# context is one level up
FROM rust:1.87 as builder

WORKDIR /app

COPY ./api/Cargo.toml ./api/Cargo.lock ./
COPY ./db /app/../db
COPY ./events /app/../events
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY ./api .

RUN cargo build --release

# 3. Финальный образ (меньше размер)
FROM debian:bookworm-slim as runtime

# Создаем директорию
WORKDIR /app

# Копируем бинарник
COPY --from=builder /app/target/release/api /app/app

# Копируем .env, если нужно
COPY .env /app/.env

# Устанавливаем порт
ENV PORT=80

# Запуск
CMD ["./app"]