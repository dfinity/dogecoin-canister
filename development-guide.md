# Development Guide

## Release Overview

This repository contains multiple packages with different release strategies:

| Package             | Versioning                                         | Published on crates.io? |
|---------------------|----------------------------------------------------|-------------------------|
| `ic-doge-canister`  | Date-based (`release/YYYY-MM-DD`) | No                      |
| `ic-doge-interface` | Semver (`X.Y.Z`)                                   | Yes                     |

### Canister IDs

**Dogecoin canister:**

| Network          | Production                    | Staging                       |
|------------------|-------------------------------|-------------------------------|
| Dogecoin Mainnet | `gordg-fyaaa-aaaan-aaadq-cai` | `bhuiy-ciaaa-aaaad-abwea-cai` |

The Dogecoin canister is deployed in production by submitting proposals to the Internet
Computer's [Network Nervous System](https://internetcomputer.org/nns).

## Releasing the Dogecoin canister

### Step 1: Create a Release PR

1. Go to Actions → Create Release PR
2. Click **Run workflow**
3. Select the canister (`ic-doge-canister`)
4. Click **Run workflow**

This creates a draft PR that updates the canister's `CHANGELOG.md` using [git-cliff](https://git-cliff.org/).

5. Review and merge the PR

### Step 2: Create GitHub Release

1. Go to Actions → Create GitHub Releases
2. Click **Run workflow**

This creates a **draft** GitHub release with:

- WASM artifact (downloaded from latest CI build on `master`)
- Candid file
- Changelog (scoped to the package's directory)
- SHA-256 checksum
- Placeholder for NNS proposal links

5. Review the draft release

### Step 3: Deploy via NNS Proposal

After the release is published:

1. Submit an NNS proposal to upgrade the canister
2. Update the release notes with the proposal link
3. Mark the release as "Latest" once deployed

## Releasing the ic-doge-interface crate

### Step 1: Create a Release PR

1. Go to Actions → Create Release PR
2. Click **Run workflow**
3. Select `library-crates`
4. Click **Run workflow**

This uses [release-plz](https://release-plz.ieni.dev/) to create a PR that:

- Bumps versions in `Cargo.toml` based on conventional commits (patch, minor, or major)
- Updates `CHANGELOG.md`

5. Review and merge the PR

### Step 2: Publish to crates.io

1. Go to Actions → Publish Crates to crates.io
2. Click **Run workflow**

This publishes the `ic-doge-interface` to crates.io and creates git tags.

## Manual WASM Build (for verification)

To manually build and verify WASM checksums:

```shell
# Clone and checkout the release commit
git clone https://github.com/dfinity/dogecoin-canister
cd dogecoin-canister
git checkout <commit-sha>

# Build reproducibly with Docker
./scripts/docker-build

# Verify checksums match the release
sha256sum *.wasm.gz
```

**Note**: Reproducible builds require Docker. There is no reproducibility guarantee on Mac M1s; preferably use Ubuntu or
Intel Macs.