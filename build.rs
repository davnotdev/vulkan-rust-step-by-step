use shaderc::*;

fn main() {
    //  (ShaderKind::Vertex, "my_shader") =>
    //  { src/my_shader.vert -> my_shader.spv }
    let shaders = [
        (ShaderKind::Vertex, "vertex"),
        (ShaderKind::Fragment, "fragment"),
    ];
    shaders.into_iter().for_each(|(kind, input)| {
        let compiler = Compiler::new().unwrap();
        let ext = match kind {
            ShaderKind::Vertex => ".vert",
            ShaderKind::Fragment => ".frag",
            _ => panic!("That shader type is not supported in build.rs."),
        };
        let input_file = String::from("src/") + input + ext;
        println!("cargo:rerun-if-changed={}", input_file);
        let input_str = std::fs::read_to_string(&input_file)
            .unwrap_or_else(|_| panic!("Could not read {}", input_file));
        let binary = compiler
            .compile_into_spirv(&input_str, kind, &input_file, "main", None)
            .unwrap_or_else(|_| panic!("Failed to compile {}.", input_file));
        let output_file = String::from(input) + ".spv";
        std::fs::write(&output_file, binary.as_binary_u8())
            .unwrap_or_else(|_| panic!("Could not write to {}", output_file));
    });
}
