#!/bin/bash

# Build the documentation using mkdocs

# Check if mkdocs is installed
if ! command -v mkdocs &> /dev/null; then
    echo "mkdocs is not installed. Installing required packages..."
    pip install mkdocs mkdocs-material mkdocs-minify-plugin mkdocs-git-revision-date-localized-plugin mkdocstrings
fi

# Move mkdocs.yml to the correct location
if [ -f "mkdocs.yml" ]; then
    cp mkdocs.yml ..
fi

# Build the documentation
cd ..
mkdocs build

# Serve the documentation if requested
if [ "$1" == "serve" ]; then
    echo "Starting local server at http://localhost:8000"
    mkdocs serve
fi

echo "Documentation built successfully in the 'site' directory"