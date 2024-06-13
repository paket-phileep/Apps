# #!/bin/bash

# # This script initializes the local environment by performing steps equivalent to the provided Dockerfile.

# # Exit immediately if a command exits with a non-zero status.
# set -e

# # Note for users
# echo "Please ensure you have sudo privileges as this script will require root access to install packages."

# # Updating and installing required packages
# echo "Updating package list and installing required packages..."
# sudo apt-get update && sudo apt-get install -y --no-install-recommends \
#     curl \
#     ca-certificates \
#     apt-transport-https \
#     gpg \
#     build-essential libcairo2-dev libpango1.0-dev libjpeg-dev libgif-dev librsvg2-dev

# # Installing NodeJS & yarn
# echo "Installing NodeJS..."
# curl -sL https://deb.nodesource.com/setup_18.x | sudo bash -

# echo "Installing yarn..."
# curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor | \
#     sudo tee /etc/apt/trusted.gpg.d/yarn.gpg
# echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
# sudo apt-get update && sudo apt-get install -y nodejs yarn

# # Setting up the working directory
# WORKDIR="/data"
# echo "Creating and changing to the working directory $WORKDIR..."
# mkdir -p $WORKDIR
# cd $WORKDIR

# # Cloning the Logseq repository
# echo "Cloning the Logseq repository..."
# git clone -b master https://github.com/logseq/logseq.git .
# echo "Repository cloned."

cd logseq

# Installing dependencies and building Logseq static resources
echo "Setting yarn network timeout..."
yarn config set network-timeout 240000 -g

echo "Installing dependencies..."
yarn install

echo "Building Logseq static resources..."
yarn release

echo "All steps completed successfully."
