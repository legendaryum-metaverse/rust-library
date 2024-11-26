#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -eou pipefail

# Function to increment version
increment_version() {
  local version=$1
  local position=$2
  IFS='.' read -ra parts <<< "$version"
  parts[position]=$((parts[position] + 1))
  for ((i = position + 1; i < ${#parts[@]}; i++)); do
    parts[$i]=0
  done
  echo "${parts[*]}" | tr ' ' '.'
}

# Get current version from Cargo.toml
current_version=$(grep '^version' legend-saga/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Increment patch version
new_version=$(increment_version "$current_version" 2)

# Update version in Cargo.toml
sed -i "s/^version = \"$current_version\"/version = \"$new_version\"/" legend-saga/Cargo.toml

# Package the crate
cargo package -p legend-saga --allow-dirty
# Publish the crate
cargo publish -p legend-saga --allow-dirty --token "$CARGO_REGISTRY_TOKEN"

echo "LAST_TAG=$new_version" >> $GITHUB_ENV
echo "Successfully published version $new_version"
