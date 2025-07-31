use std::path::Path;
use std::process::Command;
use include_dir::{include_dir, Dir};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;

fn extract_templates_to(template_dir: &Dir<'_>, dest: &Path) -> std::io::Result<()> {
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

pub fn init() {

    let root = Path::new(".tatum");

    if root.exists() {
        println!(".tatum alrady exists");
        return;
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
        println!("Please run tatum init first ");
        return;
    }
    
    let dir = root.join(&template_name);
    
    if dir.exists() {
        println!("tatum/{} already exists", &template_name);
        return;
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

pub fn to_latex(in_file_path: String, template_path: String) -> std::io::Result<()> {

    let md_path = Path::new(in_file_path.as_str());

    // Ensure the markdown file exists
    if !md_path.exists() {
        eprintln!("ERROR: Markdown file does not exist: {:?}", md_path);
        std::process::exit(1);
    }

    // Determine output .tex path
    let output_dir = md_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = md_path.file_stem()
        .expect("No file stem found")
        .to_string_lossy()
        .into_owned();
    let tex_output_path = output_dir.join(format!("{}.tex", stem));

    // Determine macros.tex path
    let macros_path = format!("{}/macros.tex", template_path);
    if !Path::new(&macros_path).exists() {
        eprintln!("ERROR: {}/macros.tex does not exist. Either run tatum compile-macros <template-path> or write your own macros.tex", template_path);
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
        .status()?; // Waits for command to finish
    
    // If the pandoc command failed
    if !status.success() {
        eprintln!("ERROR: Pandoc failed with status {:?}", status);
        std::process::exit(1);
    }

    println!("Conversion to latex completed. TEX file: {:?}", tex_output_path);

    Ok(())
}

pub fn to_pdf(in_file_path: String, template_path: String) -> std::io::Result<()> {

    let md_path = Path::new(in_file_path.as_str());

    // Ensure the markdown file exists
    if !md_path.exists() {
        eprintln!("ERROR: Markdown file does not exist: {:?}", md_path);
        std::process::exit(1);
    }

    // Determine output .pdf path
    let output_dir = md_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = md_path.file_stem()
        .expect("No file stem found")
        .to_string_lossy()
        .into_owned();
    let pdf_output_path = output_dir.join(format!("{}.pdf", stem));

    // Determine macros.tex path
    let macros_path_str = format!("{}/macros.tex", &template_path);
    let macros_path = Path::new(&macros_path_str);

    // Ensure the macros.tex file exists
    if !macros_path.exists() {
        eprintln!("ERROR: {}/macros.tex does not exist. Either run tatum compile-macros <template-path> or write your own macros.tex", template_path);
        std::process::exit(1);
    }

    // Use absolute path as we are changing directories
    let abs_macros_path = fs::canonicalize(macros_path)?;

    // Run pandoc conversion command
    let status = Command::new("pandoc")
        .arg(md_path.file_name().unwrap()) // get filename
        .arg("-o") // output flag
        .arg(pdf_output_path.file_name().unwrap()) // get filename
        .arg("--pdf-engine=pdflatex") // specify pdf engine
        .arg("-H") // header flag
        .arg(abs_macros_path) // 
        .current_dir(output_dir)
        .status()?;

    // If the pandoc command failed
    if !status.success() {
        eprintln!("ERROR: Pandoc failed with status {:?}", status);
        std::process::exit(1);
    }

    println!("Conversion to pdf completed. PDF file: {:?}", pdf_output_path);

    Ok(())
}
