# Welcome to Your Site

This is your homepage. Edit this file to customize your content.

## Getting Started

1. **Edit this file** - Modify `content/index.md` to update your homepage
2. **Add pages** - Create new `.md` files in the `content/` directory
3. **Organize content** - Use subdirectories to organize your pages
4. **Configure themes** - Edit `Sherwood.toml` to change your site theme

## Adding Content

Create new markdown files in the `content/` directory:

```
content/
├── index.md          # Your homepage
├── about.md          # About page
└── blog/
    ├── index.md      # Blog listing page
    └── first-post.md # Blog post
```

## Development

Start the development server:
```bash
sherwood dev
```

Generate your static site:
```bash
sherwood generate
```

Your site will be built in the `dist/` directory.
