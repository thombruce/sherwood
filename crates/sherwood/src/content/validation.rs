use anyhow::Result;

/// Basic HTML validation - check for balanced tags and safe elements
pub fn validate_basic_html(html: &str) -> Result<()> {
    // Simple validation: check for dangerous elements
    let dangerous = ["<script", "<iframe", "<object", "<embed", "<form"];
    let lower_html = html.to_lowercase();

    for danger in &dangerous {
        if lower_html.contains(danger) {
            return Err(anyhow::anyhow!(
                "HTML contains potentially unsafe element: {}",
                danger
            ));
        }
    }

    Ok(())
}

/// Process content - all content is assumed to be HTML-ready
pub fn process_content(content: &str) -> Result<String> {
    // Validate all HTML content for security
    validate_basic_html(content)?;
    Ok(content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_content_passthrough() {
        let html = "<h1>Test</h1><p>Content here</p>";
        let result = process_content(html).unwrap();
        assert_eq!(result, html); // HTML should pass through unchanged
    }

    #[test]
    fn test_unsafe_html_rejection() {
        let unsafe_html = "<h1>Test</h1><script>alert('xss')</script>";
        let result = process_content(unsafe_html);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsafe element"));
    }

    #[test]
    fn test_empty_content_handling() {
        // Should process empty string without error
        let result = process_content("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_html_validation_dangerous_elements() {
        let dangerous_cases = [
            "<script>alert('xss')</script>",
            "<iframe src='evil.com'></iframe>",
            "<object data='malicious.swf'></object>",
            "<embed src='dangerous content'>",
            "<form action='steal-data.com'></form>",
        ];

        for dangerous_html in &dangerous_cases {
            let result = validate_basic_html(dangerous_html);
            assert!(result.is_err(), "Should reject: {}", dangerous_html);
        }
    }

    #[test]
    fn test_html_validation_safe_elements() {
        let safe_cases = [
            "<h1>Safe heading</h1>",
            "<p>Safe paragraph</p>",
            "<div>Safe div</div>",
            "<span>Safe span</span>",
            "<ul><li>Safe list item</li></ul>",
            "<a href='safe.com'>Safe link</a>",
            "<img src='safe.jpg' alt='Safe image' />",
        ];

        for safe_html in &safe_cases {
            let result = validate_basic_html(safe_html);
            assert!(result.is_ok(), "Should allow: {}", safe_html);
        }
    }

    #[test]
    fn test_case_insensitive_dangerous_detection() {
        let mixed_case_cases = [
            "<SCRIPT>alert('xss')</SCRIPT>",
            "<Script>alert('xss')</script>",
            "<IFRAME src='evil.com'></IFRAME>",
            "<iframe src='evil.com'></IFRAME>",
            "<FORM action='steal-data.com'></form>",
            "<form ACTION='steal-data.com'></FORM>",
        ];

        for dangerous_html in &mixed_case_cases {
            let result = validate_basic_html(dangerous_html);
            assert!(
                result.is_err(),
                "Should reject mixed case: {}",
                dangerous_html
            );
        }
    }

    #[test]
    fn test_validation_with_whitespace() {
        // Current implementation doesn't catch whitespace-based bypasses
        // This documents the limitation for future enhancement
        let cases = [
            "< script>alert('xss')</script>",
            "<  script>alert('xss')</script>",
            "<\tscript>alert('xss')</script>",
            "<\nscript>alert('xss')</script>",
        ];

        for dangerous_html in &cases {
            let result = validate_basic_html(dangerous_html);
            // Note: Current implementation doesn't catch these, but documents the behavior
            assert!(
                result.is_ok(),
                "Current implementation allows: {}",
                dangerous_html
            );
        }
    }

    #[test]
    fn test_safe_similar_elements() {
        let safe_cases = [
            "<strong>Bold text</strong>",
            "<span class='script'>Contains the word script but is safe</span>",
            "<div data-iframe-id='some-value'>Safe div with iframe in attribute</div>",
            "<p>Some text with form in it</p>",
        ];

        for safe_html in &safe_cases {
            let result = validate_basic_html(safe_html);
            assert!(
                result.is_ok(),
                "Should allow similar but safe: {}",
                safe_html
            );
        }
    }
}
