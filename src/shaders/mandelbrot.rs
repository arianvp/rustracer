pub mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "shaders/mandelbrot.comp.glsl"]
    #[allow(dead_code)]
    struct Dummy;
}
