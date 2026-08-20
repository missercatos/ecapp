# ecapp — Terminal Translation Tool

A terminal translation tool written in Rust. It works both as an **interactive
shell** and as a **one-shot CLI command**, so you can translate text, files, or
even the output of other commands directly from the shell.

Supported translation backends: MyMemory (free, no key needed), Google Cloud
Translation, and DeepL.

---

## Features

- **Interactive mode** — run `ecapp` and translate from a friendly prompt, with
  dictionary mode and API configuration.
- **One-shot CLI mode** — `ecapp -t en_us zh_cn "Hello"` prints the translation
  and exits, ready for pipes like `grep`.
- **File translation** — `ecapp -t en_us zh_cn @english.txt` translates any text
  file (`.log`, `.md`, `.txt`, …) and prints it; add `-o out.txt` to write it to
  a file.
- **Command output translation** — pipe any command into ecapp:
  `man zip | ecapp -t en_us zh_cn`, `journalctl | ecapp -t auto zh_cn`, etc.
- **Default languages** — `ecapp -s en_us zh_cn` remembers your languages, so
  `ecapp -t "Hello"` works without repeating them.
- **`auto` language detection** — use `auto` as the *source* language to detect
  the input language automatically and translate to your target.
- **Performe mode (real-time translation)** — `ecapp -p` shows the translation
  of whatever you are typing at the bottom of the terminal, updating live as
  you type, delete, or edit.

---

## Installation

### Arch Linux (AUR)

```sh
# prebuilt binary
yay -S ecapp-bin

# build from git
yay -S ecapp-git
```

### From source

```sh
cargo install --path .
```

### Configuration file

ecapp stores its settings in `~/.config/ecapp/config.json` (API backend, API
key, and default languages).

---

## CLI Usage (one-shot)

```text
ecapp -h, --help                         Show help
ecapp -t <src> <tgt> <text>              Translate text
ecapp --tra=<src>-><tgt> <text>          Same as -t (also --translation=...)
ecapp --tra <src> <tgt> <text>           Same (also --translation ...)
ecapp -t <text>                          Translate using default languages
ecapp -t <src> <tgt> @file               Translate a file (log/md/txt/...)
ecapp -t <src> <tgt> @file -o out.txt    Write the result to a file
<cmd> | ecapp -t <src> <tgt>             Translate another command's output
ecapp -s <src> <tgt>                     Set default languages
ecapp --set=<src>-><tgt>                 Same
ecapp -p, --performe, --per              Real-time translation mode
ecapp -t <src> <tgt> -p                  Performe mode with explicit languages
```

### Examples

```sh
# translate a sentence
ecapp -t en_us zh_cn "Hello world"

# same, with long options
ecapp --tra=en_us->zh_cn "Hello world"
ecapp --translation=en_us->zh_cn "Hello world"

# set defaults once, then translate freely
ecapp -s en_us zh_cn
ecapp -t "Hello world"          # uses en_us -> zh_cn

# translate a file and pipe the result
ecapp -t en_us zh_cn @english.txt | grep hello

# translate a file into a new file
ecapp -t en_us zh_cn @english.txt -o chinese.txt

# translate another command's screen output (man, cat, journalctl, ...)
man zip | ecapp -t en_us zh_cn
journalctl | ecapp -t auto zh_cn

# extract the important part with grep, then translate (grep | ecapp)
grep -o "error" log.txt | ecapp -t auto zh_cn

# translate a web page fetched with curl
curl https://example.com | ecapp -t auto zh_cn

# shell output redirection works natively: > writes the translation,
# 2>&1 merges errors, and all of them can be mixed with pipes
ecapp -t en_us zh_cn "Hello" > chinese.txt
ecapp -t en_us zh_cn "@log.md" > chinese.txt 2>&1
curl https://example.com | ecapp -t auto zh_cn > zh.html 2>&1

# auto-detect the input language (only the source can be 'auto')
ecapp -t auto zh_cn "Bonjour le monde"
ecapp --set=auto->zh_cn

# real-time translation mode
ecapp -p
ecapp --performe
ecapp -t zh_cn en_us -p
```

### Pipes, redirection and large inputs

- **`<cmd> | ecapp -t <src> <tgt>`** — ecapp reads the piped data from stdin and
  prints the translation to stdout, so it can be mixed freely with other
  commands: `grep | ecapp`, `cat | ecapp`, `curl <url> | ecapp`, ...
- **`ecapp ... > file`** — output redirection works natively (the translation
  is written as plain text, without terminal colors).
- **`ecapp ... 2>&1`** — error messages and status messages go to stderr, so
  `2>&1` captures everything in one stream. The translation itself only ever
  goes to stdout.
- **Large inputs are chunked automatically** — free APIs like MyMemory limit
  each request to ~500 bytes; ecapp splits long files / piped output into
  chunks, translates them in order, and joins the results. Translation of
  whole log files or web pages works out of the box.

> **Note:** when using the `--tra=src->tgt` / `--set=src->tgt` forms, quote the
> whole argument — `ecapp '--tra=en_us->zh_cn' "Hello"`. Most shells treat the
> `>` inside the argument as an output redirection otherwise.

### `auto` language detection

`auto` may only be used as the **source** language. Non-Latin scripts (CJK,
kana, Hangul, Cyrillic, Arabic, Thai, …) are detected from the text itself;
Latin-script text is assumed to be English.

---

## Interactive Usage

Run `ecapp` with no arguments to enter the interactive shell:

```text
Welcome to ecapp — Terminal Translation Tool
Type 'help' or 'h' for available commands.

[ecapp]# help
```

### Main mode commands

| Command                 | Description                                                       |
| ----------------------- | ----------------------------------------------------------------- |
| `help`, `h`             | Show help                                                         |
| `api`                   | Manage the translation API backend and API key                    |
| `translate`, `tra`      | Enter translation mode (guided language setup)                    |
| `tra /<src> ~ <tgt>`    | Enter translation mode directly, e.g. `tra /en_us ~ zh_cn`        |
| `tra-dir`, `td`         | Dictionary-enhanced translation (bilingual dictionary + phonetics)|
| `per`, `performe`       | Real-time translation mode (needs default languages, see `set`)   |
| `set <src> <tgt>`       | Set default languages (also used by the CLI `-t` without languages)|
| `set`                   | Show the current default languages                                |
| `exit`                  | Exit ecapp                                                        |

### Translate mode commands

In translate mode, any text you type is translated. Commands start with `:/`:

| Command                   | Description                                        |
| ------------------------- | -------------------------------------------------- |
| `:/tip`                   | List all available languages                       |
| `:/reelect`, `:/ree`      | Re-select the source and target languages          |
| `:/source <code>`         | Change only the source language                    |
| `:/target <code>`         | Change only the target language                    |
| `:/swap`                  | Swap source and target languages                   |
| `:/help`, `:/h`           | Show help                                          |
| `:/exit`                  | Return to ecapp main mode                          |

### Performe mode (real-time translation)

The translation of what you are typing is shown at the bottom of the terminal
and updates live as you type or delete (debounced ~0.35s). Press **Enter** to
commit the line (prints the translation), **Esc** to leave performe mode.

Enter it with `ecapp -p` from the shell, or `per` / `performe` inside ecapp.
It needs default languages — set them with `ecapp -s` / `ecapp --set` or the
internal `set` command, or pass explicit languages with `ecapp -t zh_cn en_us -p`.

### Keyboard shortcuts

| Shortcut | Description                           |
| -------- | ------------------------------------- |
| `Ctrl+U` | Clear the current input line          |
| `Ctrl+J` | Insert a newline (multi-line input)   |
| `Ctrl+C` | Interrupt / cancel                    |

---

## Language codes

Codes use Arch-style format: lowercase with underscore (e.g. `en_us`, `zh_cn`,
`ja_jp`, `fr_fr`, `de_de`, `es_es`, `pt_br`, `ru_ru`). Run `:/tip` in translate
mode or `ecapp -h` context for the full list. `auto` is accepted as the source
language anywhere.