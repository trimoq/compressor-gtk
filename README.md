# compressor-gtk
![Main UI](doc/dnd.jpg)

A tiny utility to showcase the use of GTK+ 3 and rust on windows and linux.

This utility can resize a bunch of images selected via a file chooser of dropped by drag-and-drop.

The resize/compression settings are currently not implemented, the comanion crate [compressor](https://github.com/trimoq/compressor) supports them already.

Features:
- Drag-and-drop
- Uses GTK without directly using C or C++
- Runs on Linux and Windows
- Supports themes (compare screenshots from windows and linux below)
- Has a Gnome-Like file-chooser that nicely integrates with both supported OS
- The Windows-Release has no external dependencies except what is provided in the Zip-File

## Building
Build are run on (arch-)linux and maybe work on windows. Executables are crosscompiled to work on windows.
### For Linux
refer to [this tutorial](https://gtk-rs.org/docs-src/tutorial/cross) to set up your environment
### For Windows
Refer to 

## Usage
