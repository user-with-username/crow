#!/bin/bash

repo="user-with-username/crow"
installDir="$HOME/.crow/bin"
executableName="crow"

write_info() { echo -e "\033[36minfo:\033[0m $1"; }
write_success() { echo -e "\033[32msuccess:\033[0m $1"; }
write_action() { echo -e "\033[33maction:\033[0m $1"; }

echo -e "\n  Crow Installer" | sed 's/$/\033[34m&\033[0m/'

if [ ! -d "$installDir" ]; then
    mkdir -p "$installDir"
fi

arch=$(uname -m)
case "$arch" in
    x86_64|amd64)
        artifact="linux-x64"
        ;;
    aarch64|arm64)
        artifact="linux-arm64"
        ;;
    *)
        artifact="linux-x86"
        ;;
esac

if [[ "$OSTYPE" == "darwin"* ]]; then
    case "$arch" in
        x86_64|amd64)
            artifact="macos-x86_64"
            ;;
        aarch64|arm64)
            artifact="macos-arm64"
            ;;
        *)
            artifact="macos-x86_64"
            ;;
    esac
fi

write_info "fetching latest release metadata..."
if ! release=$(curl -s "https://api.github.com/repos/$repo/releases/latest"); then
    echo "Failed to fetch releases."
    exit 1
fi

latestTag=$(echo "$release" | grep -o '"tag_name": *"[^"]*"' | sed 's/.*"\([^"]*\)"$/\1/')

write_info "detected target ($artifact)"
write_info "downloading version $latestTag..."

downloadUrl="https://github.com/$repo/releases/download/$latestTag/$artifact"
outputPath="$installDir/$executableName"

curl -L --progress-bar "$downloadUrl" -o "$outputPath"

chmod +x "$outputPath"

write_success "crow has been installed to $installDir\n"

if [[ ":$PATH:" != *":$installDir:"* ]]; then
    shell_config=""
    if [ -n "$ZSH_VERSION" ]; then
        shell_config="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        shell_config="$HOME/.bashrc"
        [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
    else
        shell_config="$HOME/.profile"
    fi
    
    echo "" >> "$shell_config"
    echo "# Added by crow installer" >> "$shell_config"
    echo "export PATH=\"\$PATH:$installDir\"" >> "$shell_config"
    
    write_action "added crow to your PATH in $shell_config"
    echo -e "\033[90mPlease restart your terminal or run: source $shell_config\033[0m"
fi

$outputPath --version