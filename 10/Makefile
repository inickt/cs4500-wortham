.PHONY: all test clean

all: xserver xclient

xserver:
	cd Other/xserver && cargo build -j7 --release && cp target/release/xserver ../..

xclient:
	cd Other/xclient && cargo build -j7 --release && cp target/release/xclient ../..
