mod page_template;
mod render;
mod routes;
mod svg_template;

// use std::fs::read_to_string;
use std::path::PathBuf;
use std::path::Path;
use include_dir::{include_dir, Dir};
use std::process::Command;
use serde_json::Value;

use clap::{command, Parser};
use inquire::Confirm;
use render::render_doc;
use routes::construct_router;
use tokio::{
    fs,
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Serve {
        /// Whether to print logs.
        /// If true, Tatum will exclusively print out the `address:port` of the listening server once it starts.
        #[arg(short, long, default_value_t = false)]
        quiet: bool,

        /// Which port to start listening. Defaults to a random, unoccupied port assigned by the
        /// operating system.
        #[arg(short, long, default_value_t = 0)]
        port: u16,

        /// Which address to listen on.
        #[arg(short, long, default_value_t = ("127.0.0.1").to_string())]
        address: String,

        /// Specify a file path to open in a browser.
        #[arg(short, long)]
        open: Option<PathBuf>,

        /// Path to a template directory containing a page.html
        #[arg(short, long)]
        template_path: String,

    },
    Render {
        /// The location of the Markdown file to render.
        in_file: PathBuf,

        /// The path the final `HTML` file should be saved.
        /// Defaults to the same path as the `in_file`, but with the `.md` replaced with `.pdf`.
        #[arg(short, long)]
        out_file: Option<PathBuf>,

        /// Path to a template directory containing a page.html
        #[arg(short, long)]
        template_path: String,
    },
    Init,
    New {
        /// Name of template directory to create
        template_name: String,
    },
    ToLatex {
        /// Path to a template directory
        #[arg(short, long)]
        template_path: String,
    }
}

pub async fn extract_templates_to(template_dir: &Dir<'_>, dest: &Path) -> std::io::Result<()> {
    for file in template_dir.files() {
        let rel_path = file.path();
        let output_path = dest.join(rel_path);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file contents
        let contents = file.contents();
        let mut out_file = fs::File::create(&output_path).await?;
        out_file.write_all(contents).await?;
    }
    Ok(())
}

fn macro_exists(macro_name: &str) -> bool {
    let tex = format!(r#"\ifdefined{0}\typeout{{DEFINED}}\else\typeout{{UNDEFINED}}\fi\stop"#, macro_name);

    let output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(tex)
        .output()
        .expect("Failed to run pdflatex");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("DEFINED")
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args {
        // Serve option
        Args::Serve {
            quiet,
            port,
            address,
            open,
            template_path,
        } => {
            if !quiet {
                tracing_subscriber::fmt::init();
            }

            let app = construct_router(template_path);

            let listener = tokio::net::TcpListener::bind((address, port))
                .await
                .unwrap();

            if quiet {
                println!("{}", listener.local_addr().unwrap());
            } else {
                info!("Listening on {}", listener.local_addr().unwrap());
            }

            if let Some(url) = open {
                open::that(format!(
                    "http://{}?path={}",
                    listener.local_addr().unwrap(),
                    url.as_os_str().to_str().unwrap()
                ))
                .unwrap();
            }

            axum::serve(listener, app).await.unwrap();
        }
        // Render option
        Args::Render {
            mut in_file,
            out_file,
            template_path,
        } => {
            let html = render_doc(&in_file, false, template_path)
                .await
                .expect("Failed to render document.");

            let out_file = out_file.unwrap_or_else(move || {
                in_file.set_extension("html");
                in_file
            });

            if out_file.exists() {
                let ans = Confirm::new("The output file exists. Do you wish to overwrite?")
                    .with_default(false)
                    .prompt();

                match ans {
                    Ok(true) => {
                        println!("Overwriting...");
                    }
                    Ok(false) => {
                        println!("Exiting...");
                        return;
                    }
                    Err(_) => println!("Failed to recognize confirmation."),
                }
            }

            let out_file = File::create(out_file)
                .await
                .expect("Unable to open out_file.");
            let mut out_file = BufWriter::new(out_file);

            out_file
                .write_all(html.as_bytes())
                .await
                .expect("Unable to write to file.");

            out_file.flush().await.expect("Unable to write to file.");
        }
        // Init option
        Args::Init => {

            let root = Path::new(".tatum");

            if root.exists() {
                println!(".tatum alrady exists");
                return;
            }

            fs::create_dir_all(&root)
                .await
                .expect("Could not create .tatum directory");

            println!("Created .tatum directory");

            extract_templates_to(
                &include_dir!("$CARGO_MANIFEST_DIR/templates/default"),
                &root.join("default")
            ).await.expect("Could not load template `default`");

            println!("Created .tatum/default");

            extract_templates_to(
                &include_dir!("$CARGO_MANIFEST_DIR/templates/bluetot"),
                &root.join("bluetot")
            ).await.expect("Could not load template `bluetot`");

            println!("Created .tatum/bluetot");

        }
        // New option
        Args::New { template_name } => {
            
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
                .await
                .expect(format!("Could not create ./tatum/{}", &template_name).as_str());

            extract_templates_to(
                &include_dir!("$CARGO_MANIFEST_DIR/templates/default"),
                &dir
            ).await.expect("Could not copy template `default`");

            println!("Created .tatum/{}", &template_name);
        }
        // ToLatex option
        Args::ToLatex { template_path } => {

            // attempt to read file
            let path = format!("{}/katex-macros.js", &template_path);
            let content = fs::read_to_string(path)
                .await
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
                .await
                .expect("Could not create macros file");

            // loop through json
            for (k, v) in macros.as_object().unwrap() {

                // handle cases of string and array
                match v {
                    // string
                    Value::String(s) => {
                        println!("Macro: {} -> {} (no args)", k, s);
                        let command = if !macro_exists(k) {
                            format!("\\newcommand{{{}}}{{{}}}\n", k, s)
                        } else {
                            format!("\\renewcommand{{{}}}{{{}}}\n", k, s)
                        };
                        file.write_all(command.as_bytes())
                            .await
                            .expect(format!("Could not write macro {} -> {} (no args)", k, s).as_str());
                    }
                    // array
                    Value::Array(arr) if arr.len() == 2 => {
                        if let (Some(body), Some(args)) = (arr[0].as_str(), arr[1].as_u64()) {
                            println!("Macro: {} -> {} ({} args)", k, body, args);
                            let command = if !macro_exists(k) {
                                format!("\\newcommand{{{}}}[{}]{{{}}}\n", k, args, body)
                            } else {
                                format!("\\renewcommand{{{}}}[{}]{{{}}}\n", k, args, body)
                            };
                            file.write_all(command.as_bytes())
                                .await
                                .expect(format!("Could not write macro {} -> {} ({} args)", k, body, args).as_str());
                        }
                    }
                    _ => {
                        println!("Macro: {} has unknown format: {}", k, v);
                    }
                }
            }

            println!("Done!");
        }
    }
}
