# Chocolatey Package for Argus

This directory contains the Chocolatey package definition for Argus Web Crawler.

## Building the Package

To build the Chocolatey package:

```bash
# Install chocolatey (if not already installed)
# Then run:
choco pack chocolatey/argus/argus.nuspec
```

## Testing the Package

To test the package locally:

```bash
# Install from local nupkg
choco install argus -s . -y

# Test installation
argus --help

# Uninstall
choco uninstall argus -y
```

## Publishing

To publish to Chocolatey community repository:

1. Create an account at https://chocolatey.org/
2. Request to become a maintainer for the 'argus' package
3. Once approved, push the package:

```bash
choco push argus.0.1.0.nupkg
```

## Notes

- The package installs Rust and then uses cargo to install argus-crawler
- This approach ensures users get the latest compatible version
- Installation might take a few minutes due to Rust compilation
