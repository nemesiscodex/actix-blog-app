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

# Install LLVM/Clang compiler
# https://github.com/bcmyers/argonautica/tree/master/argonautica-rs#installation

# Run unit tests
cargo test

# Run the server (Add --release for an optimized build)
cargo run 
```
#### Test query:
```
{
  users {
    id
    username
    email
    bio 
    image 
    createdAt
    updatedAt
  }
}
```
Or with curl
```
curl -X POST -H "Content-Type: application/json" -d '{ "query": "{users {id username email bio image createdAt updatedAt}}" }' https://actix-blog-app.herokuapp.com/graphql -s | jq .
```
#### Will get you:
```
{
  "data": {
    "users": [
      {
        "id": "11c21a2b-e131-4b76-b32a-1872790defdb",
        "username": "user1",
        "email": "user1@example.com",
        "bio": null,
        "image": null,
        "createdAt": 1584256602,
        "updatedAt": 1584256602
      }
    ]
  }
}
```
