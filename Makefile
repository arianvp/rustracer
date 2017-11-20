.PHONY: build run

%.spv: %.glsl
	glslangValidator $< -V -S $(subst .,,$(suffix $(basename $<))) -o $@

target/testit: runtime-shader.vert.spv runtime-shader.frag.spv
	cargo build


build: target/testit
run:
	cargo run
