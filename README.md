rust-occt
===

# Pre-requisites

## Windows

- Install [Rust](https://www.rust-lang.org/tools/install)
- Install portable Build Tools or Visual Studio 2022
    - [Build Tools 2022](https://learn.microsoft.com/en-us/visualstudio/releases/2022/release-history#fixed-version-bootstrappers)
- Install [Git](https://git-scm.com/)
    - Build environment based on bash scripts. Assuming further commands executed from bash - `$`
- Install [CMake](https://cmake.org/download/)
- Install [Ninja](https://ninja-build.org/)

## MacOS

- Install [Rust](https://www.rust-lang.org/tools/install)
- Install Xcode from App Store or [Command Line Tools for Xcode](https://developer.apple.com/download/all/) and then select SDK:
```sh
$ [sudo] xcode-select --install
```
- Install [Git](https://git-scm.com/)
```sh
$ brew install git
```
- Install [CMake](https://cmake.org/download/)
```sh
$ brew install cmake
```
- Install [Ninja](https://ninja-build.org/)
```sh
$ brew install ninja
```

# Build

- Build OCCT module
```sh
$ ./occt/build-occt.sh
```
- Run
```sh
$ cargo run --release
```
> **_NOTE (Windows):_** Make sure to run build scripts from bash with MSVC environment, e.g.
```
%WINDIR%\system32\cmd.exe /c ""C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 & start "" "C:\Program Files\Git\git-bash.exe""
```
