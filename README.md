# Blub Search Engine

## Overview

Blub is a search engine project that uses Rust for indexing and searching. Go's for crawling and serving up html.

## Prerequisites

- **Go**: >= 1.23.3
- **Rust**: >= 1.79

## Project Structure
```
blub/
    crawler
    indexer
    search
```

## Getting Started

### 1. Web Crawler (Go)

- Navigate to the `crawler` directory.
- Build and execute the crawler:
```
  go build
  ./blub_crawl
```

*Note:* Crawling might take a considerable amount of time.

### 2. Indexer (Rust)

- After crawling, move the `blub1-data` directory into the `indexer` directory.
- Build and run the indexer:
```
  cd indexer
  cargo build --release
  ./target/release/blub_indexer
```

### 3. Search Server Setup (Rust & Go)

- Copy the `bulb2-data` files from the `indexer` directory to the `search` directory.
- In the `search` directory:
- Build the search engine:
  ```
  cargo build --release
  ```
- Build the web server:
  ```
  go build
  ```
- Launch the server:
  ```
  ./blub_search
  ```

## Running the Application (locally)

- Once the server is up, access the search engine by visiting `localhost:8090` in your browser.

## Development Notes

[Check out the project log](LOG.md)
