# to-html

Terminal wrapper for rendering a terminal on a website by converting ANSI escape sequences to HTML. Depends on [bash](https://www.gnu.org/software/bash/) and [ansi-to-html](https://www.npmjs.com/package/ansi-to-html).

## Examples

Execute a command:

```fish
to-html echo "Hello world"

# echo "Hello world"
```

Execute three commands:

```fish
to-html -- echo "Hello" -- echo "world" -- ls

# echo Hello
# echo world
# ls
```

Note that pipes must be escaped, otherwise they aren't passed to `to-html`:


```fish
to-html echo Hello\nworld \| grep 'H' \> somefile.txt

# echo Hello\nworld | grep 'H' > somefile.txt
```

## Installation

```fish
cargo install --git https://github.com/Aloso/colo-utils
npm i -g ansi-to-html
```
