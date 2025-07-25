mod page_template;
mod render;
mod routes;
mod svg_template;

use std::path::PathBuf;
// use std::fs;
use std::path::Path;
use include_dir::{include_dir, Dir};

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
}

/// Function to write a file to the filesystem given a path and the content to write
// async fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
//     let file = File::create(path).await?;
//     let mut writer = BufWriter::new(file);
//     writer.write_all(content.as_bytes()).await?;
//     writer.flush().await
// }

// async fn write_template(path: &Path) -> () {
//
//     // create template directory
//     fs::create_dir(&path)
//         .expect(&format!(
//             "Could not create directory {}",
//             path.to_str().unwrap_or("<invalid path>")
//         ));
//
//     let read_path = Path::new("../templates").join(
//         &path.file_name()
//     );
//
//     let page = fs::read_to_string(&read_path.join("page.html"))
//         .expect("Could not read page.html");
//     let css = fs::read_to_string(&read_path.join("style.css"))
//         .expect("Could not read style.css");
//     let macros = fs::read_to_string(&read_path.join("katex-macros.js"))
//         .expect("Could not read katex-macros.js");
//
//     let page_path = &path.join("page.html")
//     write_file(&path.join("page.html"), &page)
//         .await
//         .expect(format!("Could not write to {}", page_path.to_str()));
//
//     // write_file(
//
//
//     ()
//
//
// }

// async fn write_recursive(from: &Path, to: &Path) -> BoxFuture<'_, std::io::Result<()>> {
//     Box::pin(async move {
//         let mut dir = tokio::fs::read_dir(from).await?;
//
//         while let Some(entry) = dir.next_entry().await? {
//             let path = entry.path();
//             let dest_path = to.join(entry.file_name());
//             if path.is_dir() {
//                 write_recursive(&path, &dest_path).await?;
//             } else {
//                 let content = tokio::fs::read_to_string(&path).await?;
//                 // const FILE: &str = include_str!(&path);
//                 write_file(&dest_path, &content).await?;
//                     // .await
//                     // .expect(format!("Unable to write to {}", &path));
//             }
//         }
//         Ok(())
//     })
// }

// fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
//     fs::create_dir_all(&dst)?;
//     for entry in fs::read_dir(src)? {
//         let entry = entry?;
//         let ty = entry.file_type()?;
//         if ty.is_dir() {
//             copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
//         } else {
//             fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
//         }
//     }
//     Ok(())
// }

// fn template_path() -> PathBuf {
//     let curr_file = Path::new(file!());
//     let source_dir = curr_file.parent().unwrap();
//     source_dir.join("../templates")
// }

// static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

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

            // fs::create_dir_all(&template).expect("Could not create .tatum/default directory");
            //
            // copy_dir_all(template_path(), Path::new(".tatum"))
            //     .expect("Failed to copy templates");
            
            // // write page.html
            // const DEFAULT_PAGE: &str = include_str!("../templates/default/page.html");
            // let page_path = template.join("page.html");
            //
            // if page_path.exists() {
            //     println!(".tatum/default/page.html already exists");
            //     return;
            // }
            //
            // write_file(&page_path, DEFAULT_PAGE)
            //     .await
            //     .expect("Unable to write to ./tatum/default/page.html");
            //
            // println!("Created .tatum/default/page.html");
            //
            // // write style.css
            // const DEFAULT_CSS: &str = include_str!("../templates/default/style.css");
            // let css_path = template.join("style.css");
            //
            // if css_path.exists() {
            //     println!(".tatum/default/style.css already exists");
            //     return;
            // }
            //
            // write_file(&css_path, DEFAULT_CSS)
            //     .await
            //     .expect("Unable to write to .tatum/default/style.css");
            //
            // println!("Created .tatum/default/style.css");
            //
            // // write katex-macros.js
            // const DEFAULT_MACROS: &str = include_str!("../templates/default/katex-macros.js");
            // let macros_path = template.join("katex-macros.js");
            //
            // if macros_path.exists() {
            //     println!(".tatum/default/katex-macros.js already exists");
            //     return;
            // }
            //
            // write_file(&macros_path, DEFAULT_MACROS)
            //     .await
            //     .expect("Unable to write to .tatum/default/katex-macros.js");
            //
            // println!("Created .tatum/default/katex-macros.js");

        }
    }
}
