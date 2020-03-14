# actix-blog-app ![tests](https://github.com/nemesiscodex/actix-blog-app/workflows/tests/badge.svg)
Blog made in actix

## Requirements
- Rust
- Docker
- docker-compose

## Usage
```
# Copy example .env file
cp .env.example .env

# Run postgres
docker-compose up -d postgres

# Install diesel
cargo install diesel_cli --no-default-features --features postgres

# Run db migrations
DATABASE_URL=postgres://actix:actix@localhost:5432/actix diesel migration run

# Run unit tests
cargo test

# Run the server (Add --release for an optimized build)
cargo run 
```
```
curl -s http://localhost:8080/health
```
