.PHONY: build run

%.spv: %.glsl
	glslangValidator $< -V -S $(subst .,,$(suffix $(basename $<))) -o $@

target/testit: shaders/texture.vert.spv shaders/texture.frag.spv shaders/mandelbrot.comp.spv
	cargo build


build: target/testit
run:
	cargo run
