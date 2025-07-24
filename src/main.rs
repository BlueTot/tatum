mod page_template;
mod render;
mod routes;
mod svg_template;

use std::path::PathBuf;
use std::fs;
use std::path::Path;

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
    New,
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
        // New option
        Args::New => {

            let root = Path::new(".tatum");
            let template = root.join("default");

            if root.exists() {
                println!(".tatum alrady exists");
                return;
            }
            if template.exists() {
                println!(".tatum/default already exists");
                return;
            }

            fs::create_dir_all(&template).expect("Could not create .tatum/default directory");
            
            const DEFAULT_PAGE: &str = include_str!("../templates/page.html");
            let file_path = template.join("page.html");

            if file_path.exists() {
                println!(".tatum/default/page.html already exists");
                return;
            }

            let file = File::create(&file_path)
                .await
                .expect("Could not create .tatum/default/page.html");

            let mut writer = BufWriter::new(file);

            writer
                .write_all(DEFAULT_PAGE.as_bytes())
                .await
                .expect("Could not write to ./tatum/default/page.html");

            writer.flush().await.expect("Could not flush buffer");

            println!("Created .tatum/default/page.html");
        }
    }
}
