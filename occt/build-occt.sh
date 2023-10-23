#!/usr/bin/env bash

sh_home="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"
kBuildDir="$sh_home/build"

show_usage() {
    echo "Usage: $(basename "$0") [OPTIONS]..."
    echo "Build OpenCASCADE from sources"
    echo ""
    echo "Options"
    echo "  -h                        Show help"
    echo ""
}

while getopts 'h' opt; do
    case "$opt" in
        h) show_usage; exit 0 ;;
        *) show_usage; exit 1 ;;
    esac
done

echo_warn() {
    echo -e "\033[33m$@\033[0m" 1>&2
}

echo_err() {
    echo -e "\033[31m$@\033[0m" 1>&2
}

echo_info() {
    echo -e "\033[90m$@\033[0m" 1>&2
}

os_type() {
    if [[ $OSTYPE == "darwin"* ]]; then
        echo "osx"
    elif [[ -d /c/Windows/system32 ]]; then
        echo "win"
    else
        echo_err "Unknown OS type"
        return 1
    fi

    return 0
}

abort() {
    local ec=$?
    local msg="$@"

    [[ $msg ]] || msg="Aborted"

    echo_err "$msg"

    [[ $ec -ne 0 ]] && exit $ec || exit 1
}

git_checkout_occt() {
    [[ $# -ge 2 ]] || return 1

    local path="$kBuildDir/occt-bld"

    [[ -d "$path" ]] && \
        git -C "$path" fetch origin "$2" > /dev/null && \
        git -C "$path" checkout "$2" > /dev/null && \
        echo "$path" && \
        return 0

    rm -rf "$path" > /dev/null && \
        mkdir -p "${path%/*}" > /dev/null && \
        git clone \
            --depth 1 \
            --single-branch \
            --branch "$2" \
            "$1" \
            "$path" > /dev/null && \
        echo "$path"
}

src_path=$(git_checkout_occt \
    "https://github.com/Open-Cascade-SAS/OCCT.git" \
    "V7_7_2") || abort "Failed to checkout OpenCASCADE sources"

[[ $src_path ]] || abort "OpenCASCADE source path not specified"
[[ -d $src_path ]] || abort "OpenCASCADE source path not found"


cmake_cfg=(
    "-DCMAKE_BUILD_TYPE=Release"
    "-DBUILD_LIBRARY_TYPE:STRING=Static"
    "-DBUILD_CPP_STANDARD:STRING=C++17"
    "-DBUILD_RELEASE_DISABLE_EXCEPTIONS:BOOL=ON"
    "-DUSE_TK:BOOL=OFF"
    "-DUSE_FREETYPE:BOOL=OFF"
    "-DUSE_OPENGL:BOOL=OFF"
    "-DBUILD_MODULE_Draw:BOOL=OFF"
    "-DBUILD_MODULE_DataExchange:BOOL=OFF"
    "-DBUILD_MODULE_ModelingData:BOOL=OFF"
    "-DBUILD_MODULE_Visualization:BOOL=OFF"
)

cmake_platform_cfg=()

# case "$(os_type)" in
#     osx) cmake_platform_cfg+=("-DCMAKE_OSX_ARCHITECTURES=arm64" "-DCMAKE_OSX_DEPLOYMENT_TARGET=11")
# esac

cmake -G"Ninja" \
    -S "$src_path" \
    -B "$kBuildDir/occt-bld" \
    "${cmake_cfg[@]}" \
    "${cmake_platform_cfg[@]}" \
    -DCMAKE_INSTALL_PREFIX:PATH="$kBuildDir/occt" && \
    cmake --build "$kBuildDir/occt-bld" \
        --target "install" \
        --parallel || exit $?
