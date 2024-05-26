#!/bin/bash
NAME=vet5g-upf
export PKG_CONFIG_ALLOW_CROSS=1
export OPENSSL_STATIC=true
export OPENSSL_DIR=/musl
cargo build --target x86_64-unknown-linux-musl --release
strip target/x86_64-unknown-linux-musl/release/upf

gcc -march=native -O3 -s -static -Wall -Wextra -Werror -flto -o utils/gtp_gateway utils/gtp_gateway.c -lpthread
gcc -march=native -O3 -s -static -Wall -Wextra -Werror -flto -o utils/internet_gateway utils/internet_gateway.c -lpthread

docker build --no-cache -t ${NAME} .
docker save -o ${NAME}.tar ${NAME}
