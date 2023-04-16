fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "proto/applications.proto",
            "proto/subroutines.proto",
            "proto/subroutine_definitions.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
