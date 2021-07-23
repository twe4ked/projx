# projx

## Install

```sh
$ cargo install projx
```

### Add to your shell config

```sh
# The function name defaults to "jx"
eval "$(projx init)"

# Or customize the function name
eval "$(projx init my_func_name)"
```

## Usage

```sh
$ jx https://github.com/twe4ked/projx
# projx will clone the repo if it doesn't exist, otherwise it will `cd` there
$ cwd
# => $PROJX_DIR/github/twe4ked/projx
$ jx github/twe4ked/dotfiles
# => $PROJX_DIR/github/twe4ked/dotfiles
```
