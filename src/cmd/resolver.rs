use tera::{Context, Tera};

pub trait Resolver {
    fn resolve(&mut self, tpl: &str) -> String;
    fn add_context(&mut self, key: String, value: &str);
}

struct TeraResolver {
    context: Context,
    tera: Tera,
}

pub fn new() -> Box<dyn Resolver> {
    Box::new(TeraResolver {
        tera: Tera::default(),
        context: Context::new(),
    })
}

impl Resolver for TeraResolver {
    fn resolve(&mut self, tpl: &str) -> String {
        self
            .tera
            .render_str(tpl, &self.context)
            .expect("failed to resolve template")
    }
    fn add_context(&mut self, key: String, value: &str) {
        self.context.insert(key, value);
    }
}
