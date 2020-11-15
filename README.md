# to-html

Terminal wrapper for rendering a terminal on a website by converting ANSI escape sequences to HTML. Depends on [bash](https://www.gnu.org/software/bash/).

## Examples

Execute a command:

```bash
to-html 'echo "Hello world"'

# echo "Hello world"
```

Execute three commands:

```bash
to-html 'echo "Hello "' 'echo world' ls

# echo "Hello "
# echo world
# ls
```

Commands can contain bash syntax, including pipes and redirections:


```bash
to-html "echo Hello world! | grep 'H' > somefile.txt"
```

## Flags

  * `-n, --no-run`: Show only the prompt, don't execute the command(s)
  * `-c, --cwd`: Display the current working directory in the output


## Options

  * `-l, --highlight <commands>`: Specify comma-separated list of commands, which have subcommands that should be highlighted (e.g. `-l git,cargo,npm`)
  * `-p, --prefix <prefix>`: Prefix for CSS classes. For example, with the `to-html` prefix, the `arg` class becomes `to-html-arg`.

## TODO: Publish stylesheet
