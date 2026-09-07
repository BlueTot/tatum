# Tatum

Tatum is a simple, highly customisable CLI markdown note-taking tool. This is an extension of the original tatum, [elijah-potter/tatum](https://github.com/elijah-potter/tatum). I originally developed it for making lecture notes and submitting university assignments using _neovim_, so this tool may not be for you. Feel free to clone/fork it and add your own extensions.

## Features

### Templates

Tatum was designed to give _control_ to the user - you can customise almost everything about how your document is previewed, processed, and exported. This is done using _templates_ - preset directories that contain config files located in the `./.tatum` directory. 

The `init` command creates the `./.tatum` directory along with other files, including two default templates called `default` and `bluetot`. `default` is the original template provided by `elijah-potter`, and `bluetot` is my custom template.

```bash
tatum init
```

The command `new` creates a new template:

```bash
tatum new <TEMPLATE_NAME>
```

Each template contains at minimum these files:

- `page.html`
    * Root html file used for previewing and _exporting to HTML_.
- `style.css`
    * Custom stylesheet used for previewing and _exporting to HTML_.
- `katex-macros.js`
    * Custom list of _latex macros_ used for previewing, exporting to _HTML_, _LATEX_ and _PDF_.
- `header.tex`
    * Custom latex header used for exporting to _LATEX_ and _PDF_.

### Macros

__Katex macros__ are used to define replacements for existing latex commands to make typing easier. For example, you can alias `\mathbb{R}` to `\R`. These are specified by the user in the `katex-macros.js` file.

Visit the [official katex documentation](https://katex.org/docs/supported.html#macros) to see how to add macros yourself.

Macros are also supported when exporting to _LATEX_/_PDF_, but you have to convert the `.js` file to a `.tex` file that the conversion engine can understand. 

```bash
tatum compile-macros -t <TEMPLATE_PATH>
```

Either run the `compile-macros` command, or create the file yourself. Beware that the `compile-macros` command converts everything to a `\newcommand`, which may not work if the command is reserved. To resolve this, manually change it to a `\renewcommand`.

### More Export Formats

Often, university assignments need to be exported professionally to a _PDF_. Thats why Tatum supports exporting to _PDF_ using the `pdflatex` engine, which produces documents in a _professional latex style_. Tatum also supports converting to _latex_ using the `to-latex` command, which gives users more control over the conversion process. 

```bash
tatum to-pdf <MD_FILE_PATH> -t <TEMPLATE_PATH>
```

You can style the output _LATEX_/_PDF_ document using the `header.tex` file in each template. For example, you can add a _fancyhdr_ that shows your name, student id, and page number at the top of every page - a common university submission requirement.

Lastly, Tatum supports __bulk exporting__ to _HTML_ using the `render-all` command. It renders all files specified in the `./.tatum/render-list.json` file to their specified destinations.

## Installation

### Linux (Debian/Ubuntu)

First, install Tatum:

```bash
cargo install --git https://github.com/bluetot/tatum --locked
```

Next, install `pdflatex` and its extra packages:

```bash
sudo apt install texlive-latex-base texlive-fonts-recommended texlive-fonts-extra texlive-latex-extra
```

Install `pandoc`, which is used for file conversion:

```bash
sudo apt install pandoc
```

Initialise the `./.tatum` directory:

```bash
cd ~
tatum init
tatum compile-macros -t ./.tatum/bluetot
```

Lastly, insert the following snippet into your Neovim config:

```lua
vim.keymap.set("n", "<leader>o", function ()
   vim.fn.jobstart({
      "tatum", "serve", 
      "--open", vim.fn.expand('%'),
      "-t", vim.fn.expand('~/.tatum/bluetot')
   }, { noremap = true, silent = true })
end)
```

Alternatively, check out my _neovim_ config [here](https://github.com/BlueTot/nvim-config/public) to see how it's done.

### Windows

Tatum is expected to build and run natively on Windows without WSL. Windows builds are not currently tested by this repository's CI. The commands below use PowerShell.

#### Prerequisites

1. Install [Rust using rustup](https://rust-lang.org/tools/install/) with the default Windows MSVC toolchain. Accept the Visual Studio C++ build tools installation when prompted. If installing the build tools separately, include the C++ tools and a Windows SDK.
2. For LaTeX and PDF export, install [Pandoc for Windows](https://pandoc.org/installing.html).
3. For PDF export, also install [MiKTeX](https://miktex.org/howto/install-miktex), which provides `pdflatex`. Enable installation of missing LaTeX packages in MiKTeX.

Pandoc and MiKTeX are not needed for live HTML preview or HTML export. Open a new PowerShell window after installation and check the tools you installed:

```powershell
cargo --version
rustc --version
pandoc --version
pdflatex --version
```

#### Build and Install

From your local repository directory, compile and install Tatum:

```powershell
cd "C:\path\to\tatum"
cargo install --path . --locked
tatum --help
```

Replace the example directory with your checkout's location. The first build needs internet access to download dependencies. Cargo installs `tatum.exe` into `%USERPROFILE%\.cargo\bin`, which rustup normally adds to PATH. If PowerShell cannot find `tatum`, ensure that directory is on your user PATH and reopen PowerShell.

Alternatively, build and run without installing:

```powershell
cargo build --release --locked
.\target\release\tatum.exe --help
```

For this alternative, replace `tatum` in the examples below with the path to the built executable.

#### Live Preview and Export

From the repository directory, initialise templates once and use the existing README as an example document:

```powershell
tatum init
tatum compile-macros -t "./.tatum/bluetot"
tatum serve --open "./README.md" -t "./.tatum/bluetot"
```

The browser opens a live HTML preview that updates when you save the Markdown file. Stop the server with `Ctrl+C`. The `compile-macros` step generates `macros.tex` for LaTeX/PDF export; it is optional for HTML preview.

Run exports from the same directory:

```powershell
# Export HTML
tatum render "./README.md" -t "./.tatum/bluetot" -o "./README.html"

# Export LaTeX
tatum to-latex "./README.md" -t "./.tatum/bluetot" -o "./README.tex"

# Export PDF using Pandoc and pdflatex
tatum to-pdf "./README.md" -t "./.tatum/bluetot" -o "./README.pdf"
```

For your own documents, change to your notes directory, run `tatum init` there once, and replace `./README.md` with your Markdown file. Templates are created in the current directory's `.tatum` folder, not automatically in your home directory. You can also reuse templates by passing an absolute directory path with `-t`.

#### Usage Notes

- Quote paths, especially those containing spaces. Use `./document.md` instead of a bare `document.md` for PDF export, because the exporter changes to the input file's parent directory.
- Avoid filenames containing `&`, `#`, or `+` in live preview; the current preview URL handling does not fully encode these characters.
- PDF export uses LaTeX styling from `header.tex`, not the HTML stylesheet. Edit `.tatum/bluetot/header.tex` to replace the example name and student ID. Live preview does not automatically regenerate exported PDFs.
- If PDF export reports a missing LaTeX package, install it through MiKTeX and retry. The `bluetot` header uses `fancyhdr` and `lastpage`.
- The PDF exporter passes canonicalized header paths to Pandoc. Windows may prefix these with `\\?\`; compatibility with external tools has not been verified. If a header cannot be opened, inspect the reported path.
- The bundled HTML templates load some styles, fonts, and scripts from the internet, including in exported HTML.
