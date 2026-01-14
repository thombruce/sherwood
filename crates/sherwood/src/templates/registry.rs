use anyhow::Result;
use std::collections::HashMap;

use super::renderer::TemplateDataEnum;
use sailfish::TemplateOnce;

/// Trait for rendering templates with compile-time safety
pub trait TemplateRenderer: Send + Sync + std::fmt::Debug {
    /// Get the name of this template renderer
    fn name(&self) -> &'static str;

    /// Get the template file path this renderer handles
    fn template_path(&self) -> &'static str;

    /// Render the template with the given data
    fn render(&self, data: &TemplateDataEnum) -> Result<String>;
}

/// Trait for converting generic template data to specific template types
pub trait FromTemplateData: Sized {
    /// Convert from generic TemplateDataEnum to specific template type
    fn from(data: TemplateDataEnum) -> Self;
}

/// Registry for managing custom template renderers
#[derive(Debug)]
pub struct TemplateRegistry {
    renderers: HashMap<String, Box<dyn TemplateRenderer>>,
}

impl TemplateRegistry {
    /// Create a new empty template registry
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }

    /// Register a template renderer with the given template name
    pub fn register(
        &mut self,
        template_name: &str,
        renderer: Box<dyn TemplateRenderer>,
    ) -> Result<()> {
        if self.renderers.contains_key(template_name) {
            return Err(anyhow::anyhow!(
                "Template '{}' is already registered",
                template_name
            ));
        }

        self.renderers.insert(template_name.to_string(), renderer);
        Ok(())
    }

    /// Get a renderer for the given template name
    pub fn get_renderer(&self, template_name: &str) -> Option<&dyn TemplateRenderer> {
        self.renderers.get(template_name).map(|r| r.as_ref())
    }

    /// Check if a template is registered
    pub fn has_template(&self, template_name: &str) -> bool {
        self.renderers.contains_key(template_name)
    }

    /// Get all registered template names
    pub fn registered_templates(&self) -> Vec<String> {
        self.renderers.keys().cloned().collect()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter to bridge sailfish templates with the template registry
#[derive(Debug)]
pub struct TemplateAdapter<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TemplateAdapter<T> {
    /// Create a new template adapter
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for TemplateAdapter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TemplateRenderer for TemplateAdapter<T>
where
    T: TemplateOnce + FromTemplateData + Send + Sync + std::fmt::Debug + 'static,
{
    fn name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn template_path(&self) -> &'static str {
        // Extract template path from the template type's compile-time info
        // For now, we'll use the type name as a fallback
        // This can be improved by requiring templates to implement a path method
        std::any::type_name::<T>()
    }

    fn render(&self, data: &TemplateDataEnum) -> Result<String> {
        let template = T::from(data.clone());
        template.render_once().map_err(|e| {
            anyhow::anyhow!(
                "Failed to render template '{}': {}",
                std::any::type_name::<T>(),
                e
            )
        })
    }
}

/// Macro to register a template type with the registry
#[macro_export]
macro_rules! register_template {
    ($registry:expr, $name:expr, $template_type:ty) => {
        $registry.register(
            $name,
            Box::new($crate::templates::registry::TemplateAdapter::<$template_type>::new()),
        )
    };
}
