#!/bin/bash

# ANSI color codes
BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
CYAN="\033[0;36m"
NC="\033[0m" # No Color

echo -e "${BOLD}${BLUE}==========================================${NC}"
echo -e "${BOLD}${BLUE}  nasembler Cross-Platform Build Script  ${NC}"
echo -e "${BOLD}${BLUE}==========================================${NC}\n"

# Check if Rust and Cargo are installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed. Please install Rust and Cargo first.${NC}"
    echo -e "${YELLOW}Visit https://rustup.rs/ for installation instructions.${NC}"
    exit 1
fi

# Define target platforms with their descriptions
declare -a platforms=(
    # Linux targets
    "x86_64-unknown-linux-gnu:Linux (x86_64, GNU)"
    "x86_64-unknown-linux-musl:Linux (x86_64, musl, static)"
    "i686-unknown-linux-gnu:Linux (x86, 32-bit)"
    "aarch64-unknown-linux-gnu:Linux (ARM64)"
    "armv7-unknown-linux-gnueabihf:Linux (ARMv7)"
    
    # Windows targets
    "x86_64-pc-windows-msvc:Windows (x86_64, MSVC)"
    "x86_64-pc-windows-gnu:Windows (x86_64, MinGW)"
    "i686-pc-windows-msvc:Windows (x86, 32-bit, MSVC)"
    "i686-pc-windows-gnu:Windows (x86, 32-bit, MinGW)"
    
    # macOS targets
    "x86_64-apple-darwin:macOS (Intel x86_64)"
    "aarch64-apple-darwin:macOS (Apple Silicon M1/M2)"
    
    # FreeBSD targets
    "x86_64-unknown-freebsd:FreeBSD (x86_64)"
    "i686-unknown-freebsd:FreeBSD (x86, 32-bit)"
    
    # Other BSDs
    "x86_64-unknown-netbsd:NetBSD (x86_64)"
    "x86_64-unknown-openbsd:OpenBSD (x86_64)"
    
    # Android targets
    "aarch64-linux-android:Android (ARM64)"
    "armv7-linux-androideabi:Android (ARMv7)"
    "x86_64-linux-android:Android (x86_64)"
    "i686-linux-android:Android (x86, 32-bit)"
    
    # WebAssembly
    "wasm32-unknown-unknown:WebAssembly (wasm32)"
    
    # RISC-V
    "riscv64gc-unknown-linux-gnu:RISC-V (64-bit)"
    
    # PowerPC
    "powerpc64le-unknown-linux-gnu:PowerPC (64-bit, little-endian)"
    
    # MIPS
    "mips-unknown-linux-gnu:MIPS (32-bit, big-endian)"
    "mipsel-unknown-linux-gnu:MIPS (32-bit, little-endian)"
    
    # S390x (IBM Z)
    "s390x-unknown-linux-gnu:IBM Z (s390x)"
)

# Function to extract a short platform name (arch + OS) from a target triple
get_short_target() {
    local target="$1"
    # Get architecture (first field before '-')
    local arch="${target%%-*}"
    local os="unknown"

    if [[ "$target" == *"windows"* ]]; then
        os="windows"
    elif [[ "$target" == *"apple-darwin"* ]]; then
        os="macos"
    elif [[ "$target" == *"linux-musl"* ]]; then
        os="linux_musl"
    elif [[ "$target" == *"linux"* ]]; then
        os="linux"
    elif [[ "$target" == *"freebsd"* ]]; then
        os="freebsd"
    elif [[ "$target" == *"netbsd"* ]]; then
        os="netbsd"
    elif [[ "$target" == *"openbsd"* ]]; then
        os="openbsd"
    elif [[ "$target" == *"android"* ]]; then
        os="android"
    elif [[ "$target" == *"wasm"* ]]; then
        os="wasm"
    elif [[ "$target" == *"riscv"* ]]; then
        os="riscv"
    elif [[ "$target" == *"powerpc"* ]]; then
        os="powerpc"
    elif [[ "$target" == *"mipsel"* ]]; then
        os="mipsel"
    elif [[ "$target" == *"mips"* ]]; then
        os="mips"
    elif [[ "$target" == *"s390x"* ]]; then
        os="s390x"
    fi

    echo "${arch}_${os}"
}

# Display the list of platforms
echo -e "${BOLD}Available target platforms:${NC}\n"

for i in "${!platforms[@]}"; do
    IFS=':' read -r target description <<< "${platforms[$i]}"
    printf "${BOLD}%2d)${NC} %-35s ${CYAN}%s${NC}\n" "$((i+1))" "$target" "$description"
done

echo -e "\n${YELLOW}Note: Building for some platforms may require additional dependencies or cross-compilation toolchains.${NC}"
echo -e "${YELLOW}Some targets may require 'rustup target add <target>' to be run first.${NC}\n"

# Prompt for platform selection
echo -e "${BOLD}Enter the numbers of the platforms you want to build for (space-separated):${NC}"
read -r selections

# Convert selections to an array
IFS=' ' read -ra selected_indices <<< "$selections"
declare -a selected_targets=()

for index in "${selected_indices[@]}"; do
    # Check if the input is a valid number
    if ! [[ "$index" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Error: '$index' is not a valid number.${NC}"
        continue
    fi
    
    # Adjust for 0-based indexing and check if the index is valid
    idx=$((index - 1))
    if [ "$idx" -ge 0 ] && [ "$idx" -lt "${#platforms[@]}" ]; then
        IFS=':' read -r target _ <<< "${platforms[$idx]}"
        selected_targets+=("$target")
    else
        echo -e "${RED}Error: '$index' is not a valid platform number.${NC}"
    fi
done

# Check if any valid targets were selected
if [ ${#selected_targets[@]} -eq 0 ]; then
    echo -e "${RED}No valid platforms selected. Exiting.${NC}"
    exit 1
fi

# Create a directory for the builds
build_dir="bin"
mkdir -p "$build_dir"

# Build for each selected target
echo -e "\n${BOLD}${GREEN}Starting builds for selected platforms...${NC}\n"

for target in "${selected_targets[@]}"; do
    echo -e "${BOLD}${BLUE}=== Building for ${target} ===${NC}"
    
    # Check if the target is installed
    if ! rustup target list | grep -q "${target} (installed)"; then
        echo -e "${YELLOW}Target '$target' is not installed. Attempting to add it...${NC}"
        rustup target add "$target" || {
            echo -e "${RED}Failed to add target '$target'. Skipping...${NC}"
            continue
        }
    fi
    
    # Build for the target
    echo -e "${CYAN}Building release version...${NC}"
    cargo build --release --target="$target" || {
        echo -e "${RED}Build failed for '$target'.${NC}"
        continue
    }
    
    # Determine binary name and destination filename
    base_binary="nasembler"
    ext=""
    if [[ "$target" == *"windows"* ]]; then
        ext=".exe"
    fi

    # Use the helper function to get a short target name with arch info
    short_target=$(get_short_target "$target")
    dest_binary="${build_dir}/${base_binary}_${short_target}${ext}"

    source_path="target/$target/release/${base_binary}${ext}"
    
    # Copy the binary to the build directory with the new name
    cp "$source_path" "$dest_binary" || {
        echo -e "${RED}Failed to copy binary for '$target'.${NC}"
        continue
    }
    
    echo -e "${GREEN}Successfully built for $target!${NC}"
    echo -e "${GREEN}Binary location: ${BOLD}$dest_binary${NC}\n"
done

echo -e "${BOLD}${GREEN}Build process completed!${NC}"
echo -e "${BOLD}All binaries are available in the '${build_dir}' directory.${NC}\n"

# List successful builds
echo -e "${BOLD}${BLUE}Successfully built for:${NC}"
for target in "${selected_targets[@]}"; do
    ext=""
    if [[ "$target" == *"windows"* ]]; then
        ext=".exe"
    fi
    short_target=$(get_short_target "$target")
    dest_binary="${build_dir}/${base_binary}_${short_target}${ext}"
    if [ -f "$dest_binary" ]; then
        echo -e "${GREEN}✓${NC} $target"
    else
        echo -e "${RED}✗${NC} $target (build failed)"
    fi
done

echo -e "\n${BOLD}${BLUE}==========================================${NC}"
