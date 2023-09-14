## 使用 Rust 官方 Docker 映像
#FROM rust as builder
#
## 建立新的工作目錄
#WORKDIR /usr/src/backend
#
## 複製本地的 Cargo.toml 和 Cargo.lock 到 Docker 映像中
#COPY Cargo.toml Cargo.lock ./
#
## 複製 src 目錄到 Docker 映像中
#COPY src ./src
#
#RUN cargo update
#
## 建立 release 版本的專案
#RUN cargo build --release

# 建立新的 Docker 映像來執行專案
FROM ubuntu

# 複製從 builder 映像中編譯完成的檔案
# COPY --from=builder /usr/src/backend/target/release/slide_talker_backend /usr/local/bin/backend
RUN mkdir /app

WORKDIR /app

COPY ./target/release .

# 開放 8000 port
EXPOSE 8000

# 執行應用程式
CMD ["./slide_talker_backend"]

