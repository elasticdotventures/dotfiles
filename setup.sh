#!/bin/bash

# 🥾 migrated from promptexecution/infrastructure
# this is the *minimal* setup for _b00t_ on bash
# the first few lines can be cut and paste onto a fresh server, then run ~/.dotfiles/setup.sh

sudo apt update && sudo apt upgrade -y
sudo apt install software-properties-common -y

if ! command -v gh; then
  ## github cli (ubuntu)
  # 🤓 https://github.com/cli/cli/blob/trunk/docs/install_linux.md
  type -p curl >/dev/null || (sudo apt update && sudo apt install curl -y)
  curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
  && sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
  && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
  && sudo apt update \
  && sudo apt install git gh -y
fi

## setup extensions now.
# 🤓
gh extension install https://github.com/nektos/gh-act
# 🤓 https://github.com/github/gh-copilot
gh extension install github/gh-copilot --force



# if stow is not installed, install it
if ! command -v stow; then
  sudo apt install stow -y
fi

if [ ! -d ~/.dotfiles ]; then
  gh repo clone elasticdotventures/dotfiles ~/.dotfiles -- --depth 1
fi
# stow is idempotent
stow -d ~/.dotfiles -t ~ bash



## now setup dotfiles, and run:
## ~/.dotfiles/setup.sh



sudo apt install -y build-essential joe libnotify-bin bc

git config --global user.email "brianh@elastic.ventures"
git config --global user.name "Brian H"


sudo apt install -y ntpdate
sudo ntpdate pool.ntp.org

sudo apt update -yy
sudo apt-get install -y jq fzf ripgrep tree

# https://opentofu.org/docs/intro/install/deb
curl --proto '=https' --tlsv1.2 -fsSL 'https://packages.opentofu.org/install/repositories/opentofu/tofu/script.deb.sh?any=true' -o /tmp/tofu-repository-setup.sh
# Inspect the downloaded script at /tmp/tofu-repository-setup.sh before running
sudo bash /tmp/tofu-repository-setup.sh
rm /tmp/tofu-repository-setup.sh

sudo apt-get install -y tofu
alias tf=tofu



curl -sS https://starship.rs/install.sh | sh
echo eval "$(starship init bash)" >> ~/.bashrc


# 🦀 rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"

# dotenvy
cargo install dotenvy --bin dotenvy --features cli


# tree but ignores .git (useful for chatgpt dumps)
alias itree='rg --files | tree --fromfile'
cargo install ripgrep


# just is a command runner
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | sudo bash -s -- --to /usr/local/bin

# 💩 yq
# https://mikefarah.gitbook.io/yq/v/v3.x/
# docker run --rm -v "${PWD}":/workdir mikefarah/yq
## ubuntu noble missing ppa
# https://github.com/mikefarah/yq/issues/2081
# sudo add-apt-repository -y ppa:rmescandon/yq

#sudo apt update
#sudo apt install yq bat -y

# check for file if it exists delete it
if [ -f /etc/apt/sources.list.d/rmescandon-ubuntu-yq-noble.sources ]; then
  sudo rm /etc/apt/sources.list.d/rmescandon-ubuntu-yq-noble.sources
fi

curl https://zyedidia.github.io/eget.sh | sh
mv eget ~/.local/bin/eget
./eget mikefarah/yq --upgrade-only --tag v4.44.6


## someday..
# alias yq="podman run --rm -v \"${PWD}\":/workdir docker.io/mikefarah/yq"
# https://kislyuk.github.io/yq/

# ubuntu installs bat as batcat
sudo apt install bat -y
mkdir -p ~/.local/bin
ln -s /usr/bin/batcat ~/.local/bin/bat


## DEV workstation
# for pgrx, llvm
 sudo apt install -y libclang-dev

# aws configure
wget https://github.com/engineerd/wasm-to-oci/releases/download/v0.1.2/linux-amd64-wasm-to-oci
mv linux-amd64-wasm-to-oci wasm-to-oci
chmod +x wasm-to-oci
sudo cp wasm-to-oci /usr/local/bin

# azure
alias az="docker run -it -v ${HOME}/.ssh:/root/.ssh mcr.microsoft.com/azure-cli"

# aws
# sudo apt-get install -y awscli
cd /tmp
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
# aws configure
# OR
# mkdir -p ~/.aws
# copy config & credentials from another server

# gcloud
# https://cloud.google.com/sdk/docs/install#deb
#sudo apt-get install apt-transport-https ca-certificates gnupg curl
#curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
#echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
#sudo apt-get update && sudo apt-get install google-cloud-cli
#sudo apt-get install google-cloud-cli-gke-gcloud-auth-plugin
#gcloud init

# gcloud
# https://cloud.google.com/sdk/docs/install#deb
sudo apt-get install apt-transport-https ca-certificates gnupg curl
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
sudo apt-get update && sudo apt-get install google-cloud-cli -y
#sudo apt-get install google-cloud-cli-gke-gcloud-auth-plugin
# TODO: test for google cloud config before running init
if ! gcloud config configurations list | grep -q 'NAME'; then
  gcloud init
fi
gcloud config set compute/zone australia-southeast2-c


# The docker route probably won't work with terraform.
#docker pull gcr.io/google.com/cloudsdktool/google-cloud-cli:latest
#docker run --rm gcr.io/google.com/cloudsdktool/google-cloud-cli:latest gcloud version
#docker run -ti --name gcloud-config gcr.io/google.com/cloudsdktool/google-cloud-cli gcloud auth login
#docker run --rm --volumes-from gcloud-config gcr.io/google.com/cloudsdktool/google-cloud-cli gcloud compute instances list --project your_project



# kubectl command completion

# Installing bash completion on Linux
## If bash-completion is not installed on Linux, install the 'bash-completion' package
## via your distribution's package manager.
## Load the kubectl completion code for bash into the current shell
source <(kubectl completion bash)
## Write bash completion code to a file and source it from .bash_profile
kubectl completion bash > ~/.kube/completion.bash.inc
printf "
# kubectl shell completion
source '$HOME/.kube/completion.bash.inc'
" >> $HOME/.bash_profile
source $HOME/.bash_profile

# krew kubectl plugin
(
  set -x; cd "$(mktemp -d)" &&
  OS="$(uname | tr '[:upper:]' '[:lower:]')" &&
  ARCH="$(uname -m | sed -e 's/x86_64/amd64/' -e 's/\(arm\)\(64\)\?.*/\1\2/' -e 's/aarch64$/arm64/')" &&
  KREW="krew-${OS}_${ARCH}" &&
  curl -fsSLO "https://github.com/kubernetes-sigs/krew/releases/latest/download/${KREW}.tar.gz" &&
  tar zxvf "${KREW}.tar.gz" &&
  ./"${KREW}" install krew
)

# kubectl krew install cilium
# 💩 @jamesc says n0.

# ## kubeseal
# ## https://github.com/bitnami-labs/sealed-secrets
# KUBESEAL_VERSION='' # Set this to, for example, KUBESEAL_VERSION='0.23.0'
# curl -OL "https://github.com/bitnami-labs/sealed-secrets/releases/download/v${KUBESEAL_VERSION:?}/kubeseal-${KUBESEAL_VERSION:?}-linux-amd64.tar.gz"
# tar -xvzf kubeseal-${KUBESEAL_VERSION:?}-linux-amd64.tar.gz kubeseal
# sudo install -m 755 kubeseal /usr/local/bin/kubeseal

# Ubuntu/Debian
sudo apt-get install inotify-tools

# Rye - cargo for python
curl -sSf https://rye.astral.sh/get | RYE_INSTALL_OPTION="--yes" bash

# datafusion
cargo install datafusion-cli

