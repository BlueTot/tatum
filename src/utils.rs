use std::path::{Path, PathBuf};
use include_dir::Dir;
use std::fs;
use std::io::Write;
use colored::*;
use inquire::Confirm;
use anyhow::{Result, anyhow};

pub fn extract_templates_to(template_dir: &Dir<'_>, dest: &Path) -> std::io::Result<()> {
    for file in template_dir.files() {
        let rel_path = file.path();
        let output_path = dest.join(rel_path);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file contents
        let contents = file.contents();
        let mut out_file = fs::File::create(&output_path)?;
        out_file.write_all(contents)?;
    }
    Ok(())
}

pub fn eshow(result: Result<()>) {
    result.unwrap_or_else(|e| {
        eprintln!("{}", e.to_string());
    })
}


// Print error message for when macros.tex is not found
pub fn err_no_macro_tex(template_path: String) -> String {
    format!(
        "{} {}\n {}",
        "ERROR:".red().bold(),
        format!("{}/macros.tex does not exist.", template_path),
        "Either run `tatum compile-macros <template-path>` \
         or write your own macros.tex".yellow()
    )
}

// Print error message for when header.tex is not found
pub fn err_no_header_tex(template_path: String) -> String {
    format!(
        "{} {}\n {}",
        "ERROR:".red().bold(),
        format!("{}/header.tex does not exist.", template_path),
        "If you do not require a header, create a blank file".yellow()
    )
}

// Print error message for when markdown file is not found
pub fn err_no_md_file(md_path: &Path) -> String{
    format!(
        "{} {}",
        "ERROR:".red().bold(),
        format!("Markdown file {} does not exist", md_path.to_str().unwrap())
    )
}

// Print error message for when pandoc fails
pub fn err_pandoc_fails(status: &std::process::ExitStatus) -> String {
    format!(
        "{} {}",
        "ERROR:".red().bold(),
        format!("Pandoc failed with status {}", status)
    )
}

// Print error message
pub fn err(msg: &str) -> String {
    format!(
        "{} {}",
        "ERROR:".red().bold(),
        msg
    )
}

pub fn notify_overwrite() -> Result<()> {
    let ans = Confirm::new("The output file exists. Do you wish to overwrite?")
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => {
            println!("{}", "Overwriting ...".yellow());
            Ok(())
        }
        Ok(false) => Err(anyhow!("{}", "Exiting..".red())),
        Err(_) => Err(anyhow!("{}", "Failed to recognize confirmation.".red()))
    }
}

pub fn create_parent_directories(out_file: &PathBuf) {
    let dir_path = out_file.parent().unwrap_or_else(|| Path::new("."));
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)
            .expect(format!("Could not create directory {}", &dir_path.to_str().unwrap()).as_str());
        println!(
            "{}",
            format!(
                "Recursively created {}", &dir_path.to_str().unwrap()
            ).yellow()
        );
    }
}
