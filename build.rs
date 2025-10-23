use std::error::Error;
use ico_builder::IcoBuilder;
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    IcoBuilder::default()
        .add_source_file("./assets/icon.png")
        .build_file("./assets/icon.ico")?;

    let mut res = WindowsResource::new();
    res.set_icon("./assets/icon.ico");
    res.compile()?;

    Ok(())
}
