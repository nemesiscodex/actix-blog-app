CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table users (
    id uuid default uuid_generate_v4() primary key,
    username varchar not null unique,
    email varchar not null unique,
    password varchar not null,
    bio varchar null,
    image varchar null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create table posts (
    id uuid default uuid_generate_v4() primary key,
    author_id uuid not null,
    slug varchar not null unique,
    title varchar not null,
    description varchar not null,
    body text not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp, 
    foreign key (author_id) references users(id)
);

create table comments (
    id uuid default uuid_generate_v4() primary key,
    author_id uuid not null,
    post_id uuid not null,
    body text not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp, 
    foreign key (author_id) references users(id),
    foreign key (post_id) references posts(id)
);