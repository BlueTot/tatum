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

</br>

### Macros

__Katex macros__ are used to define replacements for existing latex commands to make typing easier. For example, you can alias `\mathbb{R}` to `\R`. These are specified by the user in the `katex-macros.js` file.

Visit the [official katex documentation](https://katex.org/docs/supported.html#macros) to see how to add macros yourself.

Macros are also supported when exporting to _LATEX_/_PDF_, but you have to convert the `.js` file to a `.tex` file that the conversion engine can understand. 

```bash
tatum compile-macros <TEMPLATE_PATH>
```

Either run the `compile-macros` command, or create the file yourself. Beware that the `compile-macros` command converts everything to a `\newcommand`, which may not work if the command is reserved. To resolve this, manually change it to a `\renewcommand`.

</br>

### More Export Formats

Often, university assignments need to be exported professionally to a _PDF_. Thats why Tatum supports exporting to _PDF_ using the `pdflatex` engine, which produces documents in a _professional latex style_. Tatum also supports converting to _latex_ using the `to-latex` command, which gives users more control over the conversion process. 

```bash
tatum to-pdf <MD_FILE_PATH> -t <TEMPLATE_PATH>
```

You can style the output _LATEX_/_PDF_ document using the `header.tex` file in each template. For example, you can add a _fancyhdr_ that shows your name, student id, and page number at the top of every page - a common university submission requirement.

Lastly, Tatum supports __bulk exporting__ to _HTML_ using the `render-all` command. It renders all files specified in the `./.tatum/render-list.json` file to their specified destinations.

## Installation

First, install Tatum:

```bash
cargo install --git https://github.com/bluetot/tatum --locked
```

Next, insert the following snippet into your Neovim config:

```lua
vim.keymap.set("n", "<leader>o", function ()
  vim.fn.jobstart({"tatum", "serve", "--open", vim.fn.expand('%')}, { noremap = true, silent = true })
end)
```

Alternatively, check out my _neovim_ config [here](https://github.com/BlueTot/nvim-config/public) to see how it's done.
