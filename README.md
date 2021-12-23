# Shot

Simple CLI that upload images to [Cloudflare Image](https://www.cloudflare.com/products/cloudflare-images/), either from clipboard, or local file.

## Install

Install from [crates.io](https://crates.io).

```bash
$ cargo install shot
```

## Usage

```
shot

USAGE:
    shot [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -d, --dry-run    Preview the command without perform any actions
    -h, --help       Print help information

SUBCOMMANDS:
    auth      Auth of Cloudflare API. Currently only supports account_id + token pair
    help      Print this message or the help of the given subcommand(s)
    paste     Upload image in clipboard to Cloudflare Image
    upload    Encode local images to PNG and upload to Cloudflare Images. For all supported
              image format, see `https://docs.rs/image/latest/image/codecs/index.html#supported-
              formats`
```
