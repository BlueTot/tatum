use std::path::{Path, PathBuf};
use std::process::Command;
use include_dir::include_dir;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use colored::*;
use crate::utils::*;

pub fn init() {

    let root = Path::new(".tatum");

    if root.exists() {
        err_dir_exists(".tatum");
        std::process::exit(1);
    }

    fs::create_dir_all(&root)
        .expect("Could not create .tatum directory");

    println!("Created .tatum directory");

    extract_templates_to(
        &include_dir!("$CARGO_MANIFEST_DIR/templates/default"),
        &root.join("default")
    ).expect("Could not load template `default`");

    println!("Created .tatum/default");

    extract_templates_to(
        &include_dir!("$CARGO_MANIFEST_DIR/templates/bluetot"),
        &root.join("bluetot")
    ).expect("Could not load template `bluetot`");

    println!("Created .tatum/bluetot");
}

pub fn new(template_name: String) {
    let root = Path::new(".tatum");

    if !root.exists() {
        eprintln!(
            "{} {}\n{}",
            "ERROR:".red().bold(),
            format!(".tatum/ directory doesn't exist."),
            "Please run `tatum init` first".yellow()
        );
        std::process::exit(1);
    }
    
    let dir = root.join(&template_name);
    
    if dir.exists() {
        err_dir_exists(dir.to_str().unwrap());
        std::process::exit(1);
    }

    fs::create_dir_all(&dir)
        .expect(format!("Could not create ./tatum/{}", &template_name).as_str());

    extract_templates_to(
        &include_dir!("$CARGO_MANIFEST_DIR/templates/default"),
        &dir
    ).expect("Could not copy template `default`");

    println!("Created .tatum/{}", &template_name);
}

pub fn compile_macros(template_path: String) {
    // attempt to read file
    let path = format!("{}/katex-macros.js", &template_path);
    let content = fs::read_to_string(path)
        .expect("Could not read katex macros");

    // strip off into a json
    let json_str = content
        .replace("window.katexMacros = ", "")
        .replace(";", "")
        .trim()
        .to_string();

    // read string to json
    let macros: Value = serde_json::from_str(&json_str).unwrap();

    // open macros file to write to
    let macros_path = format!("{}/macros.tex", &template_path);
    let mut file = File::create(macros_path)
        .expect("Could not create macros file");

    // loop through json
    for (k, v) in macros.as_object().unwrap() {

        // handle cases of string and array
        match v {
            // string
            Value::String(s) => {
                let command = format!("\\newcommand{{{}}}{{{}}}\n", k, s);
                file.write_all(command.as_bytes())
                    .expect(format!("Could not write macro {} -> {} (no args)", k, s).as_str());
                println!("Macro created: {} -> {} (no args)", k, s);
            }
            // array
            Value::Array(arr) if arr.len() == 2 => {
                if let (Some(body), Some(args)) = (arr[0].as_str(), arr[1].as_u64()) {
                    let command = format!("\\newcommand{{{}}}[{}]{{{}}}\n", k, args, body);
                    file.write_all(command.as_bytes())
                        .expect(format!("Could not write macro {} -> {} ({} args)", k, body, args).as_str());
                    println!("Macro created: {} -> {} ({} args)", k, body, args);
                }
            }
            _ => {
                println!("Macro: {} has unknown format: {}", k, v);
            }
        }
    }

    println!("Done!");
}

// Convert to latex
pub fn to_latex(
    in_file_path: String, 
    template_path: String, 
    out_file_path: Option<String>
) -> std::io::Result<()> {

    let md_path = Path::new(in_file_path.as_str());

    // Ensure the markdown file exists
    if !md_path.exists() {
        err_no_md_file(md_path);
        std::process::exit(1);
    }

    // Determine output .tex path
    let output_dir = md_path.parent().unwrap_or_else(|| Path::new("."));
    let tex_output_path = match out_file_path {
        None => {
            output_dir.join(
                md_path.file_stem().expect("No file stem found")
            ).with_extension("tex")
        }
        Some(s) => {
            PathBuf::from(&s)
        }
    };

    // Determine macros.tex path
    let macros_path = format!("{}/macros.tex", template_path);
    if !Path::new(&macros_path).exists() {
        err_no_macro_tex(template_path);
        std::process::exit(1);
    }

    // Determine header.tex path
    let header_path = format!("{}/header.tex", template_path);
    if !Path::new(&header_path).exists() {
        err_no_header_tex(template_path);
        std::process::exit(1);
    }

    // Run pandoc conversion command
    let status = Command::new("pandoc")
        .arg(md_path)
        .arg("-s") // standalone flag
        .arg("-o") // output flag
        .arg(&tex_output_path)
        .arg("-H") // header flag
        .arg(macros_path)
        .arg("-H") // second header flag
        .arg(header_path)
        .status()?; // Waits for command to finish
    
    // If the pandoc command failed
    if !status.success() {
        err_pandoc_fails(&status);
        std::process::exit(1);
    }

    println!("Conversion to latex completed. TEX file: {:?}", tex_output_path);

    Ok(())
}

// Convert to pdf
pub fn to_pdf(
    in_file_path: String, 
    template_path: String,
    out_file_path: Option<String>
) -> std::io::Result<()> {

    let md_path = Path::new(in_file_path.as_str());

    // Ensure the markdown file exists
    if !md_path.exists() {
        err_no_md_file(md_path);
        std::process::exit(1);
    }

    // Determine output .pdf path
    let output_dir = md_path.parent().unwrap_or_else(|| Path::new("."));
    let pdf_output_path = match out_file_path {
        None => {
            output_dir.join(
                md_path.file_stem().expect("No file stem found")
            ).with_extension("pdf")
        }
        Some(s) => {
            PathBuf::from(&s)
        }
    };

    // Determine macros.tex path
    let macros_path_str = format!("{}/macros.tex", &template_path);
    let macros_path = Path::new(&macros_path_str);

    // Ensure the macros.tex file exists
    if !macros_path.exists() {
        err_no_macro_tex(template_path);
        std::process::exit(1);
    }

    // Determine header.tex path
    let header_path_str = format!("{}/header.tex", &template_path);
    let header_path = Path::new(&header_path_str);

    // Ensure the header.tex file exists
    if !header_path.exists() {
        err_no_header_tex(template_path);
        std::process::exit(1);
    }

    // Use absolute path as we are changing directories
    let abs_macros_path = fs::canonicalize(&macros_path)?;
    let abs_header_path = fs::canonicalize(&header_path)?;

    // Handle the pdf output path separetly as the path may not exist yet
    let abs_pdf_output_path = if pdf_output_path.is_absolute() {
        pdf_output_path.clone()
    } else {
        std::env::current_dir()?.join(&pdf_output_path)
    };

    // Run pandoc conversion command
    let status = Command::new("pandoc")
        .arg(md_path.file_name().unwrap()) // get filename
        .arg("-o") // output flag
        .arg(abs_pdf_output_path) // get filename
        .arg("--pdf-engine=pdflatex") // specify pdf engine
        .arg("-H") // header flag
        .arg(abs_macros_path) // macros path (absolute)
        .arg("-H") // second header flag
        .arg(abs_header_path) // header path (absolute)
        .current_dir(output_dir)
        .status()?;

    // If the pandoc command failed
    if !status.success() {
        err_pandoc_fails(&status);
        std::process::exit(1);
    }

    println!("Conversion to pdf completed. PDF file: {:?}", pdf_output_path);

    Ok(())
}
