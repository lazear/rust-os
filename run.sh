#!/bin/bash
cargo run --bin builder && \
qemu-system-x86_64 \
	./build/disk.img \
	-monitor stdio \
	-serial file:serial.txt \
	-d cpu_reset
