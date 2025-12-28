use std::fs;
use std::path::{Path, PathBuf};
use pulldown_cmark::{html, Parser, Options};
use anyhow::Result;

pub struct SiteGenerator {
    input_dir: PathBuf,
    output_dir: PathBuf,
}

impl SiteGenerator {
    pub fn new(input_dir: &Path, output_dir: &Path) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub async fn generate(&self) -> Result<()> {
        // Clean output directory
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)?;
        }
        fs::create_dir_all(&self.output_dir)?;

        // Find all markdown files
        let markdown_files = self.find_markdown_files(&self.input_dir)?;
        
        if markdown_files.is_empty() {
            println!("No markdown files found in {}", self.input_dir.display());
            return Ok(());
        }

        // Process each markdown file
        for file_path in markdown_files {
            self.process_markdown_file(&file_path).await?;
        }

        println!("Site generated successfully in {}", self.output_dir.display());
        Ok(())
    }

    fn find_markdown_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if !dir.exists() {
            println!("Content directory {} does not exist", dir.display());
            return Ok(files);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                files.extend(self.find_markdown_files(&path)?);
            } else if let Some(extension) = path.extension() {
                if extension == "md" || extension == "markdown" {
                    files.push(path);
                }
            }
        }
        
        Ok(files)
    }

    async fn process_markdown_file(&self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let relative_path = file_path.strip_prefix(&self.input_dir)?;
        
        // Convert .md to .html
        let html_path = self.output_dir.join(relative_path).with_extension("html");
        
        // Create parent directories if needed
        if let Some(parent) = html_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Extract title from first h1 or use filename
        let title = self.extract_title(&content)
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });

        // Convert markdown to HTML with semantic structure
        let html_content = self.markdown_to_semantic_html(&content)?;
        
        // Generate complete HTML document
        let full_html = self.generate_html_document(&title, &html_content);
        
        fs::write(&html_path, full_html)?;
        println!("Generated: {}", html_path.display());
        
        Ok(())
    }

    fn extract_title(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return Some(trimmed[2..].trim().to_string());
            }
        }
        None
    }

    fn markdown_to_semantic_html(&self, markdown: &str) -> Result<String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        
        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        Ok(self.enhance_semantics(&html_output))
    }

    fn enhance_semantics(&self, html: &str) -> String {
        let mut enhanced = html.to_string();
        
        // Wrap paragraphs in semantic sections if they seem like articles
        enhanced = self.wrap_articles(&enhanced);
        
        // Add semantic structure to lists
        enhanced = self.enhance_lists(&enhanced);
        
        enhanced
    }

    fn wrap_articles(&self, html: &str) -> String {
        // Simple heuristic: if content has multiple headings, wrap in article tags
        let heading_count = html.matches("<h").count();
        if heading_count > 1 {
            format!("<article>\n{}\n</article>", html)
        } else {
            html.to_string()
        }
    }

    fn enhance_lists(&self, html: &str) -> String {
        // Convert plain lists to more semantic versions when appropriate
        html.replace("<ul>", "<ul class=\"content-list\">")
            .replace("<ol>", "<ol class=\"numbered-list\">")
    }

    fn generate_html_document(&self, title: &str, content: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            color: #333;
        }}
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 2rem;
            margin-bottom: 1rem;
            color: #222;
        }}
        h1 {{
            border-bottom: 2px solid #eee;
            padding-bottom: 0.5rem;
        }}
        code {{
            background: #f4f4f4;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
        }}
        pre {{
            background: #f4f4f4;
            padding: 1rem;
            border-radius: 5px;
            overflow-x: auto;
        }}
        pre code {{
            background: none;
            padding: 0;
        }}
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 1rem;
            color: #666;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 0.5rem;
            text-align: left;
        }}
        th {{
            background: #f9f9f9;
        }}
        article {{
            margin-bottom: 2rem;
        }}
    </style>
</head>
<body>
    <main>
        {content}
    </main>
</body>
</html>"#,
            title = title,
            content = content
        )
    }
}

pub async fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let generator = SiteGenerator::new(input_dir, output_dir);
    generator.generate().await
}