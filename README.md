# colo-utils

Wrapper around `colo` that emits HTML which can be displayed on a website.

## Example

Execute `colo s orange` and emit HTML:

```fish
colo-utils s orange
```

Execute three commands and emit HTML:

```fish
colo-utils -- s ff3377 -- s orange -- s 'hsv(300, 100%, 100%)'
```

## Installation

```fish
cargo install --git https://github.com/Aloso/colo-utils
npm i -g ansi-to-html
```
