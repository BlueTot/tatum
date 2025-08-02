mod page_template;
mod render;
mod routes;
mod svg_template;
mod commands;
mod utils;

use crate::commands::{to_html, init, new, compile_macros, to_latex, to_pdf, render_all};

use std::path::PathBuf;

use clap::{command, Parser};
use routes::construct_router;
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
        template: String,

    },
    Render {
        /// The location of the Markdown file to render.
        in_file: PathBuf,

        /// The path the final `HTML` file should be saved.
        /// Defaults to the same path as the `in_file`, but with the `.md` replaced with `.html`.
        #[arg(short, long)]
        out_file: Option<PathBuf>,

        /// Path to a template directory containing a page.html
        #[arg(short, long)]
        template: String,

        /// Whether to create parent directory of output file
        #[arg(short)]
        parent: bool,
    },
    Init,
    New {
        /// Name of template directory to create
        template_name: String,
    },
    CompileMacros {
        /// Path to a template directory
        #[arg(short, long)]
        template: String,
    },
    ToLatex {
        /// Path to Markdown file to render.
        in_file: String,

        /// Path to a template directory
        #[arg(short, long)]
        template: String,

        /// The path the final `LATEX` file should be saved
        /// Defaults to the same path as the `in_file`, but with the `.md` replaced with a `.tex`
        #[arg(short, long)]
        out_file: Option<String>,

        /// Whether to create parent directory of output file
        #[arg(short, long)]
        parent: bool

    },
    ToPdf {
        /// Path to Markdown file to render
        in_file: String,

        /// Path to a template directory
        #[arg(short, long)]
        template: String,

        /// The path the final `PDF` file should be saved.
        /// Defaults to the same path as the `in_file`, but with the `.md` replaced with `.pdf`.
        #[arg(short, long)]
        out_file: Option<String>,

        /// Whether to create parent directory of output file
        #[arg(short, long)]
        parent: bool
    },
    RenderAll {
        /// Path to a template directory
        #[arg(short, long)]
        template: String,

        /// Whether to create parent directory of output file
        #[arg(short)]
        parent: bool
    }
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args {
        // Serve option - async
        Args::Serve {
            quiet,
            port,
            address,
            open,
            template,
        } => {
            if !quiet {
                tracing_subscriber::fmt::init();
            }

            let app = construct_router(template);

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
        // Render option - async
        Args::Render { in_file, out_file, template, parent } => {
            to_html(in_file, out_file, template, parent)
                .await
                .expect("Failed to render");
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
        Args::CompileMacros { template } => {
            compile_macros(template)
        }
        // ToLatex option - compiles to a latex.
        // Used to give more control to user
        Args::ToLatex { in_file, template, out_file, parent } => {
            match to_latex(in_file, template, out_file, parent) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e.to_string())
            }
        }
        // ToPdf option - compiles to a pdf
        Args::ToPdf { in_file, template, out_file, parent } => {
            match to_pdf(in_file, template, out_file, parent) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e.to_string())
            }
        }
        // RenderAll option - renders all the files in the render-list.json file
        Args::RenderAll {template, parent} => {
            match render_all(template, parent).await {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e.to_string())
            };
        }
    }
}

