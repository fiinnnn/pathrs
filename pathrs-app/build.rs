fn main() -> Result<(), Box<dyn std::error::Error>> {
    let release = std::env::var("PROFILE") == Ok("release".into());

    spirv_builder::SpirvBuilder::new("../pathrs-shader", "spirv-unknown-vulkan1.2")
        .release(release)
        .print_metadata(spirv_builder::MetadataPrintout::Full)
        .build()?;

    Ok(())
}
