use minijinja::{Environment, Error, path_loader};
use minijinja_autoreload::AutoReloader;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct TemplateEngine {
    reloader: Arc<AutoReloader>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self, Error> {
        let reloader = AutoReloader::new(move |notifier| {
            let template_path = PathBuf::from("templates");
            let mut env = Environment::new();
            env.set_loader(path_loader(&template_path));

            // only set up file watching (with debug enabled) if in development
            #[cfg(debug_assertions)]
            {
                env.set_debug(true);
                notifier.watch_path(&template_path, true);
            }

            Ok(env)
        });

        Ok(TemplateEngine {
            reloader: Arc::new(reloader),
        })
    }

    pub fn render(&self, template_name: &str, context: minijinja::Value) -> Result<String, Error> {
        let env = self.reloader.acquire_env()?;
        let template = env.get_template(template_name)?;
        template.render(context)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize template engine")
    }
}
