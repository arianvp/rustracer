.PHONY: build run

%.spv: %.glsl
	glslangValidator $< -V -S $(subst .,,$(suffix $(basename $<))) -o $@

target/testit: shaders/runtime-shader.vert.spv shaders/runtime-shader.frag.spv
	cargo build


build: target/testit
run:
	cargo run
