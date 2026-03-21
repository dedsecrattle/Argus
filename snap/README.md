# Snap Package for Argus

This directory contains the Snap package definition for Argus Web Crawler.

## Building the Snap

To build the snap package, you need snapcraft installed:

```bash
# On Ubuntu/Debian
sudo apt install snapcraft

# Build the snap
snapcraft

# This will create argus_0.1.0_amd64.snap
```

## Testing the Snap

To test the snap locally:

```bash
# Install the snap (in devmode for testing)
sudo snap install --devmode argus_0.1.0_amd64.snap

# Test installation
argus --help

# Uninstall
sudo snap remove argus
```

## Publishing to Snap Store

1. Register for a Snap Store account at https://snapcraft.io/
2. Register the name 'argus':
   ```bash
   snapcraft register argus
   ```

3. Upload and release:
   ```bash
   snapcraft upload --release=stable argus_0.1.0_amd64.snap
   ```

## Installation from Store

Once published, users can install with:
```bash
sudo snap install argus
```

## Notes

- The snap includes all dependencies including the Rust toolchain
- Initial size will be large (~200MB) due to Rust compilation
- Future versions can use stage-packages to reduce size
