mod page_template;
mod render;
mod routes;
mod svg_template;
mod commands;

use crate::commands::{init, new, compile_macros, to_latex, to_pdf};

use std::path::PathBuf;

use clap::{command, Parser};
use inquire::Confirm;
use render::render_doc;
use routes::construct_router;
use tokio::{
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
    CompileMacros {
        /// Path to a template directory
        #[arg(short, long)]
        template_path: String,
    },
    ToLatex {
        /// Path to Markdown file to render.
        in_file_path: String,

        /// Path to a template directory
        #[arg(short, long)]
        template_path: String,
    },
    ToPdf {
        /// Path to Markdown file to render
        in_file_path: String,

        /// Path to a template directory
        #[arg(short, long)]
        template_path: String,
    }
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
            init()
        }
        // New option
        Args::New { template_name } => {
            new(template_name.to_string())
        }
        // CompileMacros option
        Args::CompileMacros { template_path } => {
            compile_macros(template_path)
        }
        // ToLatex option - compiles to a latex.
        // Used to give more control to user
        Args::ToLatex { in_file_path, template_path } => {
            to_latex(in_file_path, template_path)
                .expect("Failed to convert to latex");
        }
        // ToPdf option - compiles to a pdf
        Args::ToPdf { in_file_path, template_path } => {
            to_pdf(in_file_path, template_path)
                .expect("Failed to convert to latex");
        }
    }
}
