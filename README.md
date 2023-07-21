# txt2epub

A simple cli tool that can convert some specific txt files to epub files.

## Usage

```bash
$ txt2epub --help

Convert TXT file to Epub

Usage: txt2epub [OPTIONS] [FILES]...

Arguments:
  [FILES]...  The Files those need to be convert into epub

Options:
  -o, --out-dir <OUT_DIR>  Output directory
  -h, --help               Print help
```

## Build

```bash
git clone <this repo>
cd txt2epub
cargo build --release
```

## Support Structure

### For novel metadata like title, author, etc.

use yaml to parse

### For novel content

#### part

```regex
^第.+[部|卷] (.*)$
```

For default, txt2epub will treat the content between the line which match the regex and the chapter regex as the preface of the part. It will be centered. If you don't want to center the preface, you can add `[LongPreface]` in the line after the part title line.

Also, if your file doesn't have any part, txt2epub will treat whole chapter as a part which will not show the part page.

#### chapter

```regex
^第.+[章] (.*)$
```