# adot

A minimal dotfile manager written in Rust, inspired by [dotdrop](https://github.com/deadc0de6/dotdrop).

> **Warning: This is AI slop, use at your own risk!**

## Install

### Homebrew (macOS)

```bash
brew install Dimfred/tap/adot
```

### Cargo (all platforms)

```bash
cargo install --locked --git https://github.com/Dimfred/adot.git
```

### AUR (Arch Linux)

```bash
yay -S adot-bin
# or
paru -Sy adot-bin
```

## Usage

```bash
# install dotfiles for current hostname
adot install -c path/to/config.yaml

# install with explicit profile
adot install -c path/to/config.yaml -p my-profile

# silent mode (errors still print)
adot install -c path/to/config.yaml -s
```

Config is auto-discovered in this order if `-c` is not given:
1. `$XDG_CONFIG_HOME/adot/config.yaml`
2. `~/.config/adot/config.yaml`
3. `~/.adot/config.yaml`

Profile defaults to the machine hostname if `-p` is not given.

## Config

```yaml
# dotfiles section — define what to manage
dotfiles:
  # link: single absolute symlink (default type)
  f_zshrc:
    dst: ~/.zshrc              # where it gets installed
    src: home/.zshrc           # relative to dotpath (default: dotfiles/)
    type: link

  # link_children: create dst dir, symlink each child individually
  # useful for ~/.config/ where each app gets its own symlink
  d_config:
    dst: ~/.config/
    src: config/
    type: link_children

  # copy: copy files/dirs into dst (merges, never removes existing content)
  d_pulse:
    dst: ~/.config/pulse
    src: config_c/pulse
    type: copy

  # template: render {{@@ var @@}} and conditionals, then copy
  d_git:
    dst: ~/.config/git
    src: config_t/git
    type: template

# profiles section — group dotfiles per machine
profiles:
  # base profiles for inheritance
  server_base:
    dotfiles:
      - f_zshrc
      - d_config

  # machine profile — matched by hostname
  my-laptop:
    dotfiles:              # dotfiles specific to this machine
      - d_pulse
      - d_git
    include:               # inherit from other profiles
      - server_base
    variables:             # template variables (override global)
      git:
        email: user@example.com
      editor: nvim
      colors:
        foreground: "#02B0C7"
        background: "#000000"

# global variables — available to all profiles
variables:
  editor: vim              # default, overridden by profile

# dynamic variables — value is stdout of shell command
dynvariables:
  hostname: "hostname -s"
  os: "uname -s"
```

## Template Syntax

Templates use dotdrop-compatible syntax:

```
# variable substitution
email = {{@@ git.email @@}}

# conditional blocks (if / elif / endif)
{%@@ if profile == "work-laptop" @@%}
proxy = http://proxy:8080
{%@@ elif profile == "home-pc" @@%}
proxy =
{%@@ endif @@%}

# comments (stripped from output)
{#@@ This line won't appear in the output @@#}
```

## Dotfile Types

| Type | Behavior |
|------|----------|
| `link` | Absolute symlink: `dst -> src` |
| `link_children` | Create `dst/` dir, symlink each child in `src/` individually |
| `copy` | Copy files/dirs into `dst` (merges with existing content) |
| `template` | Render variables + conditionals, then copy result |

## License

MIT
