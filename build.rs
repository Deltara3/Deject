use std::env;
use std::error::Error;
use ico_builder::IcoBuilder;
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    IcoBuilder::default()
        .add_source_file("./assets/icon.png")
        .build_file("./assets/icon.ico")?;

    let mut res = WindowsResource::new();
    
    if let Ok(value) = env::var("DEJECT_WINDRES") {
        res.set_windres_path(&value);
    }

    res.set_icon("./assets/icon.ico");
    res.compile()?;

    Ok(())
}
