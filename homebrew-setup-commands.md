# Homebrew Setup Commands

After creating the repository on GitHub, run:

```bash
cd ~/homebrew-argus
git remote add origin https://github.com/dedsecrattle/homebrew-argus.git
git branch -M main
git push -u origin main
```

## For Users to Install:

```bash
# Add the tap
brew tap dedsecrattle/argus

# Install argus
brew install argus

# Run it
argus --version
```

## To Update the Formula Later:

```bash
cd ~/homebrew-argus
# Edit Formula/argus.rb with new version and SHA256
git commit -am "Update argus to v0.1.1"
git push
```
