# Active CD 

This rust tool is a supplement to the default cd command, by visually displaying the subdirectories and assigning keybinds.

## Installation

`cargo install active-cd`

This installs the binary, in order to move the terminal's working directory on exit you need to add the following to your .bashrc:
```bash
cda() {
    cd "$(active-cd)"
}
```

## Usage

Upon calling the command you are presented with a list of subdirectories. You can press their presented keybinds to move to them, as well:
-   `q` - quit and move to selected path
-   `esc` - quit and don't move
-   `u` - move up a directory
-   `~` - move to home directory
-   `/` - move to root directory

