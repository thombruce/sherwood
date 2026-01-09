+++
title = "Deployment Guide"
date = "2024-01-18"
page_template = "docs.stpl"
+++

# Deployment Guide

This guide covers various ways to deploy your Sherwood-generated static site.

## Static Hosting Options

### Netlify

Netlify is ideal for JAMstack sites with continuous deployment:

1. **Connect Repository:**
   - Sign up for Netlify account
   - Connect your Git repository
   - Configure build settings

2. **Build Settings:**
   ```
   Build command: cargo run -- generate
   Publish directory: dist
   ```

3. **Environment Variables:**
   Set `RUST_VERSION` to `1.70.0` or higher

### Vercel

Vercel offers zero-config deployment:

1. **Install Vercel CLI:**
   ```bash
   npm i -g vercel
   ```

2. **Configure Project:**
   Create `vercel.json`:
   ```json
   {
     "buildCommand": "cargo run -- generate",
     "outputDirectory": "dist",
     "installCommand": ""
   }
   ```

3. **Deploy:**
   ```bash
   vercel --prod
   ```

### GitHub Pages

Free static hosting for public repositories:

1. **Create Workflow:**
   `.github/workflows/deploy.yml`:
   ```yaml
   name: Deploy
   on:
     push:
       branches: [main]
   jobs:
     deploy:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v2
         - uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
         - run: cargo run -- generate
         - uses: peaceiris/actions-gh-pages@v3
           with:
             github_token: ${{ secrets.GITHUB_TOKEN }}
             publish_dir: ./dist
   ```

2. **Enable GitHub Pages:**
   - Repository Settings → Pages
   - Source: GitHub Actions

### Cloudflare Pages

Fast CDN-backed static hosting:

1. **Connect Repository**
2. **Build Settings:**
   ```
   Build command: cargo run -- generate
   Build output directory: dist
   ```
3. **Deploy**

## Custom Server Deployment

### Simple HTTP Server

For development or simple hosting:

```bash
# Using Python
cd dist && python -m http.server 8000

# Using Node.js
npx serve dist

# Using Rust basic-server
cargo install basic-http-server
basic-http-server dist
```

### Nginx

Production-grade web server configuration:

```nginx
server {
    listen 80;
    server_name example.com;
    root /var/www/sherwood/dist;
    index index.html;

    # Handle clean URLs
    location / {
        try_files $uri $uri.html $uri/index.html =404;
    }

    # Cache static assets
    location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### Apache

Web server configuration:

```apache
<VirtualHost *:80>
    ServerName example.com
    DocumentRoot /var/www/sherwood/dist
    
    # Enable clean URLs
    RewriteEngine On
    RewriteCond %{REQUEST_FILENAME} !-f
    RewriteCond %{REQUEST_FILENAME} !-d
    RewriteRule ^(.*)$ $1.html [L]
    
    RewriteCond %{REQUEST_FILENAME}/ -d
    RewriteRule ^(.*)$ $1/index.html [L]
</VirtualHost>
```

## Build Optimization

### Reduce Build Time

For large sites, consider these optimizations:

1. **Parallel Processing:**
   Future versions may support parallel markdown processing

2. **Selective Rebuild:**
   Only regenerate changed content

3. **Asset Optimization:**
   ```bash
   # Minify CSS
   npm install -g clean-css-cli
   find dist -name "*.css" -exec cleancss -o {} {} \;
   
   # Optimize images
   npm install -g imagemin-cli
   imagemin dist/images/* --out-dir=dist/images
   ```

### Caching Strategy

Implement proper caching headers:

```nginx
# HTML files - short cache for content updates
location ~* \.html$ {
    expires 1h;
    add_header Cache-Control "public, must-revalidate";
}

# Static assets - long cache
location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

## CI/CD Integration

### Docker

Containerized deployment:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM nginx:alpine
COPY --from=builder /app/target/release/sherwood /usr/local/bin/
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Kubernetes

Deployment manifest:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sherwood-site
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sherwood-site
  template:
    metadata:
      labels:
        app: sherwood-site
    spec:
      containers:
      - name: sherwood
        image: sherwood-site:latest
        ports:
        - containerPort: 80
```

## Performance Tips

1. **Use CDN** for static asset delivery
2. **Enable Gzip/Brotli** compression
3. **Implement proper caching headers**
4. **Optimize images** and use modern formats
5. **Minimize bundle sizes** through content optimization

Your Sherwood site is now ready for production deployment!