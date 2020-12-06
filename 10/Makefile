.PHONY: all xserver xclients

all: xserver xclients

xserver:
	cd Other/xserver && cargo build -j7 --release && cp target/release/xserver ../..

xclients:
	cd Other/xclients && cargo build -j7 --release && cp target/release/xclients ../..
