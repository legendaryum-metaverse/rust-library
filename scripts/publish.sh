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
current_version=$(grep '^version' my-awesome-rabbitmq-lib/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Increment patch version
new_version=$(increment_version "$current_version" 2)

# Update version in Cargo.toml
sed -i "s/^version = \"$current_version\"/version = \"$new_version\"/" my-awesome-rabbitmq-lib/Cargo.toml

# Package the crate
cargo package -p my-awesome-rabbitmq-lib --allow-dirty

#cargo publish -p my-awesome-rabbitmq-lib --allow-dirty --token "$CARGO_REGISTRY_TOKEN"
secret=Y2lvSngyUGNhZHZHQng2Zm9oaFBlYTRYY2lMQTc0ZkhudW4=
# Publish the crate "$CARGO_REGISTRY_TOKEN"
cargo publish -p my-awesome-rabbitmq-lib --allow-dirty --token "$(echo $secret | base64 -d)"

echo "LAST_TAG=$new_version" >> $GITHUB_ENV
echo "Successfully published version $new_version"
