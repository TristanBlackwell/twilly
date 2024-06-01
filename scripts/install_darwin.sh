#!/bin/bash

# Define the directory where you want to install twilly
INSTALL_DIR="/usr/local/bin"


# prepare the download URL
GITHUB_LATEST_VERSION=$(curl -L -s -H 'Accept: application/json' https://github.com/TristanBlackwell/twilly/releases/latest | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
GITHUB_FILE="twilly_cli-${GITHUB_LATEST_VERSION//v/}-x86_64-apple-darwin.tar.gz"
GITHUB_URL="https://github.com/TristanBlackwell/twilly/releases/download/${GITHUB_LATEST_VERSION}/${GITHUB_FILE}"

# install/update the local binary
curl -L -o twilly_cli.tar.gz $GITHUB_URL
tar xzvf twilly_cli.tar.gz twilly_cli
install -Dm 755 twilly_cli -t "$INSTALL_DIR"
rm twilly_cli twilly_cli.tar.gz

echo "twilly_cli has been installed successfully."
