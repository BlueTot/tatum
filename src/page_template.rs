// use askama::Template;

// #[derive(Debug, Template)]
// #[template(path = "page.html")]
// pub struct PageTemplate {
//     pub title: String,
//     pub body: String,
//     pub use_websocket: bool,
// }

use tera::{Context, Tera};

pub trait Renderable {
    fn template_path(&self) -> &str;
    fn context(&self) -> Context;

    fn render(&self) -> anyhow::Result<String> {
        let mut tera = Tera::default();
        tera.add_template_file(self.template_path(), Some("template"))?;
        tera.render("template", &self.context()).map_err(Into::into)
    }
}

pub struct PageTemplate {
    pub title: String,
    pub body: String,
    pub use_websocket: bool,
    pub template_path: String,
}

impl Renderable for PageTemplate {
    fn template_path(&self) -> &str {
        &self.template_path
    }

    fn context(&self) -> Context {
        let mut ctx = Context::new();
        ctx.insert("title", &self.title);
        ctx.insert("body", &self.body);
        ctx.insert("use_websocket", &self.use_websocket);
        ctx
    }
}
