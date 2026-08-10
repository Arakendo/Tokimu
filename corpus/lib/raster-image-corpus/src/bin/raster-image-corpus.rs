use std::path::PathBuf;

use raster_image_corpus::RasterCaseFilter;

fn main() -> Result<(), String> {
    let filter = parse_filter(std::env::args().skip(1))?;
    let output = PathBuf::from("target/raster-image-corpus/review-v1");
    let artifacts = raster_image_corpus::write_selected_artifacts_with_filter(&output, &filter)?;
    println!(
        "wrote {} raster review files to {}",
        artifacts.len(),
        output.display()
    );
    Ok(())
}

fn parse_filter(args: impl Iterator<Item = String>) -> Result<RasterCaseFilter, String> {
    let mut filter = RasterCaseFilter::default();
    let mut args = args;
    while let Some(argument) = args.next() {
        if argument == "--help" {
            return Err(usage());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} requires a value"))?;
        match argument.as_str() {
            "--format" => filter.format = Some(value),
            "--feature" => filter.feature = Some(value),
            "--expected" => filter.expected = Some(value),
            "--stage" => filter.expected_stage = Some(value),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    Ok(filter)
}

fn usage() -> String {
    "usage: raster-image-corpus [--format png|jpeg|bmp] [--feature NAME] [--expected candidate-pass|candidate-rejection] [--stage decode|profile]".to_owned()
}
