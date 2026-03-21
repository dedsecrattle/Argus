#!/bin/bash

# Update all Cargo.toml files with publishing metadata

REPO=${1:-"dedsecrattle/argus"}

# Function to update metadata for a crate
update_crate_metadata() {
    local crate_name=$1
    local crate_path=$2
    local description=$3
    local keywords=$4
    local categories=$5
    
    echo "Updating $crate_name..."
    
    # Create temporary file
    temp_file=$(mktemp)
    
    # Read the Cargo.toml and add metadata after edition line
    awk -v crate="$crate_name" -v desc="$description" -v keywords="$keywords" -v categories="$categories" -v repo="$REPO" '
    /^[[]package[]]/ {
        in_package = 1
        print
        next
    }
    in_package && /^name = / {
        print
        next
    }
    in_package && /^version = / {
        print
        next
    }
    in_package && /^edition = / {
        print
        print "authors = [\"Argus Contributors\"]"
        print "description = \"" desc "\""
        print "homepage = \"https://github.com/" repo "/argus\""
        print "repository = \"https://github.com/" repo "/argus\""
        print "documentation = \"https://docs.rs/" crate "\""
        print "readme = \"../../README.md\""
        print "license = \"MIT\""
        print "keywords = [" keywords "]"
        print "categories = [" categories "]"
        in_package = 0
        next
    }
    { print }
    ' "$crate_path/Cargo.toml" > "$temp_file"
    
    # Replace original
    mv "$temp_file" "$crate_path/Cargo.toml"
}

# Update all crates
update_crate_metadata "argus-common" "crates/argus-common" "Common types and utilities for the Argus web crawler" "\"crawler\", \"web\", \"common\", \"types\"" "\"web-programming\""

update_crate_metadata "argus-config" "crates/argus-config" "Configuration management for the Argus web crawler" "\"crawler\", \"config\", \"settings\"" "\"config\", \"web-programming\""

update_crate_metadata "argus-dedupe" "crates/argus-dedupe" "Content deduplication utilities for web crawling" "\"crawler\", \"deduplication\", \"simhash\", \"bloom\"" "\"web-programming\", \"algorithms\""

update_crate_metadata "argus-fetcher" "crates/argus-fetcher" "HTTP fetching utilities with retry logic for web crawling" "\"crawler\", \"http\", \"fetch\", \"retry\"" "\"web-programming\", \"network-programming\""

update_crate_metadata "argus-frontier" "crates/argus-frontier" "URL frontier implementations for web crawling" "\"crawler\", \"frontier\", \"queue\", \"redis\"" "\"web-programming\", \"development-tools\""

update_crate_metadata "argus-parser" "crates/argus-parser" "HTML and sitemap parsing utilities for web crawling" "\"crawler\", \"html\", \"parser\", \"sitemap\"" "\"parsing\", \"web-programming\""

update_crate_metadata "argus-robots" "crates/argus-robots" "Robots.txt parsing and caching for web crawling" "\"crawler\", \"robots\", \"txt\", \"parsing\"" "\"parsing\", \"web-programming\""

update_crate_metadata "argus-storage" "crates/argus-storage" "Storage backends for crawled web data" "\"crawler\", \"storage\", \"s3\", \"filesystem\"" "\"database\", \"web-programming\""

update_crate_metadata "argus-worker" "crates/argus-worker" "Worker implementation for distributed web crawling" "\"crawler\", \"worker\", \"distributed\", \"async\"" "\"web-programming\", \"asynchronous\""

echo "All metadata updated!"
