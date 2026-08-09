fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", hello_alpha_policy::study_report_json()?);
    Ok(())
}
