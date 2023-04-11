fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["proto/applications.proto", "proto/subroutines.proto"],
        &["proto"],
    )?;
    Ok(())
}
