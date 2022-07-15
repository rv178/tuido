# TUIdo

Simple TUI based to-do app written in Rust.

TUIdo stores a JSON file that contains your todos in `~/.config/todos.json`.

![Alt](https://media.discordapp.net/attachments/985433521084563486/997110251226681405/unknown.png)

### Installation

#### Arch Linux

```
cd contrib/
makepkg -si
```

You can compile it using `cargo` or install [baker](https://github.com/rv178/baker) and install it like this:

```
bake setup
bake
sudo bake install
```

A binary will be copied to `./bin/tuido`

### Uninstalling

```
sudo bake uninstall
```

### Usage

```
tuido
```
