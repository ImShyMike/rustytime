use minijinja::{Environment, Error};
use std::sync::Arc;

#[cfg(feature = "dev-templates")]
use minijinja::{path_loader};
#[cfg(feature = "dev-templates")]
use minijinja_autoreload::AutoReloader;
#[cfg(feature = "dev-templates")]
use std::path::PathBuf;

#[derive(Clone)]
pub struct TemplateEngine {
    #[cfg(feature = "dev-templates")]
    reloader: Arc<AutoReloader>,
    #[cfg(not(feature = "dev-templates"))]
    env: Arc<Environment<'static>>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self, Error> {
        #[cfg(feature = "dev-templates")]
        {
            let reloader = AutoReloader::new(move |notifier| {
                let template_path = PathBuf::from("templates");
                let mut env = Environment::new();
                env.set_loader(path_loader(&template_path));
                env.set_debug(true);
                notifier.watch_path(&template_path, true);
                Ok(env)
            });

            Ok(TemplateEngine {
                reloader: Arc::new(reloader),
            })
        }

        #[cfg(not(feature = "dev-templates"))]
        {
            let mut env = Environment::new();
            
            // embed templates as static strings for production
            env.add_template("admin_dashboard.html", include_str!("../../templates/admin_dashboard.html"))?;
            env.add_template("dashboard.html", include_str!("../../templates/dashboard.html"))?;
            env.add_template("home.html", include_str!("../../templates/home.html"))?;

            Ok(TemplateEngine {
                env: Arc::new(env),
            })
        }
    }

    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: minijinja::Value) -> Result<String, Error> {
        #[cfg(feature = "dev-templates")]
        {
            let env = self.reloader.acquire_env()?;
            let template = env.get_template(template_name)?;
            template.render(context)
        }

        #[cfg(not(feature = "dev-templates"))]
        {
            let template = self.env.get_template(template_name)?;
            template.render(context)
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize template engine")
    }
}
