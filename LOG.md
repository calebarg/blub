# Project Log

## 11/30/2024:

- **Choosing a Web Crawler**
  - **Open Source Web Crawlers**: Found a repository listing various web crawlers. Interpreted languages were disqualified immediately. The C/C++ categories were lacking a bit, which left Golang.

    (Go is definitely a language suited for crawling. It makes sense it would have a good selection to choose from)

    - **PuerkitoBio/gocrawl**: Ended up choosing this crawler. Got a test crawler up and running.

- **Scraping: DAY 1**

  At first glance, I want the URL, the title of the "document", and its contents. I can get the URL easily. And I can naively get the title and content with the title and body HTML tags, assuming the document is HTML.

  - **Issues**:
    - Not all documents are HTML, so this needs addressing.
    - Filtering certain content types. A simple blacklist seems to work for now, though it will need expansion.

  - **Storage**:

    I'll save each document like this into `<id>.blub1` where the id is a running count.

    ```text
    Byte ordering LE
    | U16 - url len   | <-- Header
    | U16 - title len |
    | U32 - body len  |
    -------------------
    | url             | <-- Data
    | title           |
    | body            |
    ```

    Each `.blub1` file is stored under a directory labeled with the domain name it was scraped from, e.g., `blub1-data/<domain_name>/<id>.blub1`.

- **Indexing: DAY 1**

  - **Tantivy**: The obvious choice for indexing, given the project constraints. I know next to zero Rust since I did a bit of Rustlings ~4 years ago...

  - **Setup**: While my crawler was running, I set up Rust on my machine and started exploring tantivy-cli to get a handle on things. Tantivy-cli was straightforward to get running, achieving a searchable index in under 4 minutes without writing any Rust.

  - **Crawler Issue**: The crawler had been downloading hundreds of `.phar` files (PHP archive files). Added a blacklist rule for these.

  - **Rust Progress**: Moving slowly in Rust, but I've got a program that can deserialize all the blub1 files from the crawler.

- **Schemas: DAY 1**

  My current schema:

  - `url`, STRING | STORED
  - `title`, TEXT | FAST | STORED
  - `body`, TEXT | FAST | STORED

## 12/01/2024:

- **Crawling: DAY 2**

  - **Normalization and Concurrency**: Normalized URLs and tried to make the crawler concurrent via Go routines. However, gocrawl kept crashing when used with goroutines, so I modified it to start multiple instances of itself, dividing the source domains among them.

  - **Tuning**: Spent time tuning gocrawl options for better crawling speed.

- **Indexing: DAY 2**

  - **Environment Issues**: Too cold in my workspace, so switched to my laptop, ran into build errors due to Rust version differences. Adjusted the Cargo file to use Tantivy 0.22.0 with Rust 1.79.0.

  - **Progress**: Didn't accomplish as much as hoped due to various issues. Also, there was a problem with the blub1 serialization/deserialization step.

## 12/02/2024:

- **Indexing: DAY 3**

  Fixed the deserialization issue with blub1 files. Now successfully creating and storing an index.

- **Blub Search: DAY 3**

  - **Webserver**: Need a webserver for displaying results and allowing searches. Since using Go for the webserver, I had to get Go calling into Rust:

    - Compiled a static library in Rust with C linkage and no name mangling.
    - Got blub_search.rs to load an index and perform queries.

  - **Search Functionality**: Initially planned to use Tantivy's explanation context to extract relevant blurbs, but for now, just extracting URL and Title fields, serializing to JSON for Go.

  - **Performance**: Longer queries exceed the allotted 50ms latency. Fortunately, this is in debug build; release build might help.

  - **Tantivy Settings**: Investigated "max search time" in Tantivy, capped the size of indexed blub1 file contents.

- **Scraping: DAY 3**

  - **Content Type Detection**: Tried whitelists on URL extensions but realized it wasn't effective. Now using `http.DetectContentType` in gocrawl's `Visit` function to check for UTF-8 HTML content.

## 12/03/2024:

- **Indexing: DAY 4**

  Updated the indexer to track the number of pages indexed per domain, a project requirement.

- **Search: DAY 4**

  Discovered Tantivy's `SnippetGenerator` and cried tears of joy.
