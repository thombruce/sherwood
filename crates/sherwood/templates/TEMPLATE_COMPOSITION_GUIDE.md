# Template Composition with ContentItem

Sherwood now provides a clean, ergonomic API for template composition through the `crate::partials::ContentItem` struct.

## Usage in Custom Templates

### Basic Usage
```sailfish
<!-- In your custom template at docs/templates/my_list.stpl -->
<div class="my-custom-list">
    <% for item in items { %>
        <%+ ContentItem::from(item) %>
    <% } %>
</div>
```

### Alternative Ergonomic Options
```sailfish
<!-- Using From trait conversion (most ergonomic) -->
<%+ item.clone().into() %>

<!-- Using explicit constructor -->
<%+ ContentItem::new(item) %>
```

## ContentItem Structure

`ContentItem` wraps `ListItemData` which contains:
- `title: String` - Item title
- `url: String` - Relative URL path
- `date: Option<String>` - Publication date (if available)
- `excerpt: Option<String>` - Item excerpt (if available)

## Template Variables

Within `content_item.stpl`, you have access to:
- `item.title` - Item title
- `item.url` - Relative URL
- `item.date` - Optional date string
- `item.excerpt` - Optional excerpt string

## Example Custom Template

```sailfish
<!-- docs/templates/blog_card.stpl -->
<article class="card">
    <header>
        <h3><a href="/<%= item.url %>"><%= item.title %></a></h3>
        <% if let Some(date) = item.date { %>
            <time class="post-date"><%= date %></time>
        <% } %>
    </header>
    <% if let Some(excerpt) = item.excerpt { %>
        <p class="excerpt"><%- excerpt %></p>
    <% } %>
</article>
```

## Benefits

✅ **Clean API**: `ContentItem::from(data)` is self-documenting  
✅ **Type Safety**: Strong typing with proper encapsulation  
✅ **Ergonomic**: Multiple usage patterns supported  
✅ **Discoverable**: Available in `crate::partials` module  
✅ **Reusable**: Single source of truth for item rendering  

This makes it easy for users to create custom templates while maintaining clean separation of concerns.