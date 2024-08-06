export APP_IMAGE_NAME=whiplash

unittest:
	cargo test

build:
	docker build --progress=plain -t ${APP_IMAGE_NAME} . --build-arg CONFIG_PATH="./config.yaml"

run:
    docker run whiplash

compile:
	cargo build --release