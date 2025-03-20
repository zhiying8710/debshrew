# Debshrew Documentation

This directory contains the documentation for the Debshrew project. The documentation is built using [MkDocs](https://www.mkdocs.org/) with the [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) theme.

## Building the Documentation

To build the documentation, run the following command:

```bash
./build.sh
```

This will create a `site` directory in the project root with the built documentation.

## Serving the Documentation Locally

To serve the documentation locally, run:

```bash
./build.sh serve
```

This will start a local server at http://localhost:8000.

## Documentation Structure

- `index.md`: The main entry point for the documentation
- `installation.md`: Installation guide
- `quickstart.md`: Quick start guide
- `configuration.md`: Configuration guide
- `architecture.md`: Architecture overview
- `cdc-concepts.md`: CDC concepts
- `metashrew-integration.md`: Metashrew integration
- `wasm-transform-guide.md`: WASM transform guide
- `api/`: API reference
  - `debshrew.md`: Debshrew API reference
  - `debshrew-runtime.md`: Debshrew Runtime API reference
  - `debshrew-support.md`: Debshrew Support API reference
- `mkdocs.yml`: MkDocs configuration file
- `build.sh`: Script to build the documentation

## Contributing to the Documentation

To contribute to the documentation:

1. Make your changes to the Markdown files
2. Build the documentation locally to verify your changes
3. Submit a pull request

## Deployment

The documentation is automatically built and deployed to GitHub Pages when changes are pushed to the main branch. The deployment is handled by the GitHub Actions workflow defined in `.github/workflows/docs.yml`.