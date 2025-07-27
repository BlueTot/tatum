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
    pub css: String,
    pub macros: String,
    pub use_websocket: bool,
    pub template_path: String,
}

impl Renderable for PageTemplate {
    fn template_path(&self) -> &str {
        &self.template_path
    }

    fn context(&self) -> Context {
        let mut ctx = Context::new();

        // insert the title
        ctx.insert("title", &self.title);

        // insert the body
        ctx.insert("body", &self.body);

        // insert whether to use websockets for updating
        ctx.insert("use_websocket", &self.use_websocket);

        // insert the corresponding css file
        ctx.insert("inline_css", &self.css);
        
        // insert the corresponding katex macros file
        ctx.insert("katex_macros", &self.macros);

        ctx
    }
}
