install:
	pip3 install cargo-lambda

format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet
run:
	cargo run 

release:
	cargo lambda build --release

release-arm:
	cargo lambda build --release --arm64

deploy:
	cargo lambda deploy

all: format lint test run