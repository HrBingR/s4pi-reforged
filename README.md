# s4pi-reforged 

s4pi-reforged is a tool for merging custom content and mods for The Sims 4 in the .package format, written in Rust with threading to be as fast as possible.

## Features

The tool supports the following features:

- Merging .package files into one
- Unmerging already-merged .package files
- Extracting thumbnails from .package files, whether merged or unmerged
- Full compatibility with packages created or merged by Sims 4 Studio; packages merged with s4pi-reforged can be opened by S4S, and vice versa.
- Blazing fast merging and unmerging of packages
- Automatic package and file conflict detection
- Automatic compression

## Introduction

This tool was largely inspired by both the Sims 4 Studio application, as well as [s4pe](https://github.com/s4ptacle/Sims4Tools).

The goal here was to provide a way to safely but very quickly merge a lot of packages for The Sims 4, while preserving the ability to unmerge, and to provide compatibility with Sims 4 Studio in doing so.

Further to this, unlike the Sims 4 Studio application, s4pi-reforged has native support for Linux, while retaining support for Windows. Mac support is planned for the future.

## Attributions

While none of the existing code and implementation was directly used, a lot of the work to develop this tool relied on C# libraries and wrappers written for and used in s4pi, which forms part of [s4pe](https://github.com/s4ptacle/Sims4Tools). None of this would have been possible without the fine work done by the original developers.

### Contributors

#### s4pi/s4pe

* Peter Jones - Main author of s3pe/s3pi
* [Rick] - a pioneer in TS4
* [ChaosMageX] - Initial s4p* setup; work on DATA, RIG and GEOM wrappers
* [andrewtavera] - Mesh parts and other help
* [granthes] - Several contributions pre-release and in the early stages
* [snaitf] - Decoding and contributions for CCOL, COBJ, trims as well as bugfixes
* [IngeJones] - a kind lady who doesn't want her name mentioned
* [Kuree] - Maintained the project in 2014 and 2015
* [CmarNYC] - current contributions see [here] (https://github.com/s4ptacle/Sims4Tools/commits/develop?author=cmarNYC)
* [pbox] - current contributions see [here] (https://github.com/s4ptacle/Sims4Tools/commits/develop?author=pboxx)
* [Buzzler] - current contributions see [here] (https://github.com/s4ptacle/Sims4Tools/commits/develop?author=BrutalBuzzler)

#### s4pi-reforged

* [HrBingR](https://github.com/HrBingR) - Me

## Use

### Download

Download the latest version of s4pi-reforged from the releases section.

### Usage

The app is really easy to open and use, functioning largely the same on Linux and Windows, and can be used in one of three ways.

#### GUI

This mode uses a graphical user interface (GUI), which makes function and package selection really easy.

To open the app, simply double-click the s4pi-reforged (Linux) or s4pi-reforged.exe (Windows) file, and the GUI app will launch.

At the bottom will be buttons to merge, unmerge, and extract thumbnails, with the main screen providing console output to indicate progress and completion.

- **Merge:**
  - Navigate to and select the folder with your unmerged package files, and wait for the console window to indicate that the merging is complete.
  - The merged package will be in a new 'merged' subfolder in the same folder you provided.
- **Unmerge:**
  - Navigate to and select the merged package file that you want to unmerge, and wait for the console window to indicate that unmerging is complete.
  - The unmerged packages will be in a new 'unmerged' subfolder in the same folder as the package you provided.
- **Extract > Thumbnails:**
  - Navigate to and select the package file you want to extract thumbnails from. This can be a merged or unmerged package file.
  - The extracted thumbnails will be in a new 'thumbs' subfolder in the same folder as the package you provided.

Note: Windows users may need to add the S4PI_FORCE_GUI=1 [environment variable](https://pureinfotech.com/create-custom-environment-variables-windows-10/?utm_source=chatgpt.com) if the TUI opens instead of the GUI when launching by double clicking.

#### TUI

This mode launches a terminal user interface, for users more familiar with the terminal but don't want to have to type out long commands and arguments.

Simply launch s4pi-reforged or s4pi-reforged.exe from a terminal or command-line app, and you'll be presented with these options:

1. Merge
2. Unmerge
3. Extract > 1. Images

Simply select an option by typing the corresponding number, and pressing enter.

Instructions for each operation work largely the same as in the GUI, so reference the above section if needed.

Note: Windows users may need to add the S4PI_FORCE_TUI=1 [environment variable](https://pureinfotech.com/create-custom-environment-variables-windows-10/?utm_source=chatgpt.com) to force the TUI if the GUI launches instead when opening the app via a command window.

#### CLI

This mode allows you to use the app through a command line interface, for users that excel at working in a terminal, and don't mind typing long paths, or simply for users on a headless system.

##### CLI Arguments:

`--help`: This argument displays the available commands and usage. This argument can be used with any of the below commands to see usage information, and in some cases subcommands.

`merge`: This command takes one argument, which is the path to the folder containing the packages you want to merge. Merged package will be in a new 'merged' subfolder in the same folder you provided.

`unmerge`: This command takes one argument, which is the path to the merged package file you wish to unmerge. The unmerged packages will be in a new 'unmerged' subfolder in the same folder as the package you provided.

`extract thumbnails`: This command takes one argument, which is the path to the package file (merged or unmerged) for which you want to extract thumbnails. Extracted thumbnails will be in a new 'thumbs' subfolder in the same folder as the package you provided.

##### CLI Examples

```
s4pi-reforged merge /home/SomeUser/SomeFolderWithPackages

s4pi-reforged.exe merge "C:\Users\SomeUser\Documents\SomeFolderWithPackages"

s4pi-reforged unmerge /home/SomeUser/SomeFolder/SomeCC.package

s4pi-reforged.exe unmerge "C:\Users\SomeUser\Documents\SomeCC.package"

s4pi-reforged extract thumbnails /home/SomeUser/SomeFolder/SomeCC.package

s4pi-reforged.exe extract thumbnails "C:\Users\SomeUser\Documents\SomeCC.package"
```

## Disclaimer

Sims 4 Studio is not open source, and no code from Sims 4 Studio has been disassembled, decompiled, or reverse engineered in the development of s4pi-reforged.

