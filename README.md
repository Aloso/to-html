# colo-utils

To generate the images on https://aloso.github.io/colo/, I previously used screenshots from alacritty. With this tool, I can now run

```fish
FORCE_ANSI_OUTPUT=1 colo s orange | colo-utils dim | ansi-to-html
```

to convert the ANSI escape sequences to HTML. `ansi-to-html` is a NPM package that unfortunately ignores the dimmed style, hence I developed `colo-utils`. Install it with

```fish
cargo install --git https://github.com/Aloso/colo-utils
npm i -g ansi-to-html
```
