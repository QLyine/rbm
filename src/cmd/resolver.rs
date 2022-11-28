use tera::{Context, Tera};

pub trait Resolver {
    fn resolve(&mut self, tpl: &str) -> String;
    fn add_context(&mut self, key: String, value: String);
}

struct TeraResolver {
    context: Context,
    tera: Tera
}

pub fn new() -> Box<dyn Resolver> {
    return Box::new(TeraResolver{tera: Tera::default(), context: Context::new() })
}

impl Resolver for TeraResolver {
    fn resolve(&mut self, tpl: &str) -> String {
        return self.tera.render_str(tpl, &self.context).expect("failed to resolve template");
    }
    fn add_context(&mut self, key: String, value: String) {
        self.context.insert(key, value.as_str());
    }
}