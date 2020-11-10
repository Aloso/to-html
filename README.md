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

You can pass environment variables to commands without displaying them, by using the `-e` flag:

```fish
to-html -e colo s ff3377 \| pastel desaturate .2 \| FORCE_ANSI_OUTPUT=1 colo s

#  executed: colo s ff3377 | pastel desaturate .2 | FORCE_ANSI_OUTPUT=1 colo s
# displayed: colo s ff3377 | pastel desaturate .2 | colo s
```

## Installation

```fish
cargo install --git https://github.com/Aloso/colo-utils
npm i -g ansi-to-html
```
