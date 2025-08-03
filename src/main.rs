mod page_template;
mod render;
mod routes;
mod svg_template;
mod commands;
mod utils;

use crate::commands::{to_html, init, new, compile_macros, to_latex, to_pdf, render_all};
use crate::utils::eshow;

use std::path::PathBuf;

use clap::{command, Parser};
use routes::construct_router;
use tracing::info;

#[derive(Parser, Debug)]
#[command(
    version, 
    about, 
    long_about = "A simple CLI tool that serves, renders, and exports markdown files with custom templates"
)]
enum Args {
    /// Starts a webserver to serve .md files as HTML
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
    /// Renders a .md file to HTML
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
    /// Creates the tatum config directory
    Init,
    /// Creates a new template directory
    New {
        /// Name of template directory to create
        template_name: String,
    },
    /// Compiles a given template's katex-macros.json to macros.tex. Must be run before exporting
    /// to PDF or LATEX
    CompileMacros {
        /// Path to a template directory
        #[arg(short, long)]
        template: String,
    },
    /// Exports a .md file to LATEX
    ToLatex {
        /// Path to Markdown file to render.
        in_file: String,

        /// Path to a template directory
        #[arg(short, long)]
        template: String,

        /// The path the final `LATEX` file should be saved.
        /// Defaults to the same path as the `in_file`, but with the `.md` replaced with a `.tex`
        #[arg(short, long)]
        out_file: Option<String>,

        /// Whether to create parent directory of output file
        #[arg(short, long)]
        parent: bool

    },
    /// Exports a .md file to PDF using the pdflatex engine
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
    /// Renders all files specified in ./.tatum/render-list.json to their specified locations to
    /// HTML
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
            eshow(to_html(in_file, out_file, template, parent).await);
        }
        // Init option
        Args::Init => { 
            eshow(init());
        }
        // New option
        Args::New { template_name } => {
            eshow(new(template_name.to_string()));
        }
        // CompileMacros option
        Args::CompileMacros { template } => {
            eshow(compile_macros(template));
        }
        // ToLatex option - compiles to a latex.
        // Used to give more control to user
        Args::ToLatex { in_file, template, out_file, parent } => {
            eshow(to_latex(in_file, template, out_file, parent));
        }
        // ToPdf option - compiles to a pdf
        Args::ToPdf { in_file, template, out_file, parent } => {
            eshow(to_pdf(in_file, template, out_file, parent));
        }
        // RenderAll option - renders all the files in the render-list.json file
        Args::RenderAll {template, parent} => {
            eshow(render_all(template, parent).await);
        }
    }
}

