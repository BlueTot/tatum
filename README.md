# Tatum

An extension of [elijah-potter/tatum](https://github.com/elijah-potter/tatum) used to improve markdown note-taking in the terminal by providing styling templates, macros and better exports.

## Features

- **Runtime Templating**
    * Ability to choose *custom* styling templates at runtime via the `-t` option.
- **Latex Macros**
    * Easy customisation of *latex replacement macros* to make typing easier.
- **Better Exporting**
    * Bulk exporting of `.md` to `.html` via the `render-all` command
    * Professional `.pdf` exports using the `pdflatex` pdf-engine, with support for custom `.tex` header files
    * Export to _latex_ for further control over PDF exporting pipeline

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

## Features

Tatum aims to make entirely self-contained `HTML` files.
If you reference an image in your Markdown, Tatum will resolve the location of the image, encode it as a data URL, and place it in the final file.
