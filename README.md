Disclaimer - This is a "fork" of the project on crates.io (which currently has no repo listed). I took the source from the cargo cache on my machine

A Rust preprocessor for [mdBook](https://github.com/rust-lang/mdBook), replacing regular expression patterns with specified strings. Regex syntax is based on the [Regex](https://crates.io/crates/regex) Rust crate.

## Usage

Install the crate

```
cargo install mdbook-regex
```

Specify the patterns and string replacement in a `.json` file

```json
[
    {
        "pattern" : "Your Pattern",
        "template" : "Your Template"
    }
]
```

Add the preprocessor to your `book.toml` file, and specify the path of the `.json` patterns file

```toml
[preprocessor.regex]
patterns = "path/to/patterns.json"
```

## Example

The following pattern

```json
[
    {
        "pattern" : "``collapse:(?P<title>([^\n])*)\n(?P<content>(?s:.)*)\n``",
        "template" : "<details>\n<summary>${title}</summary>\n<div class='collapsed'>\n${content}\n</div>\n</details>"
    }
]
```

allows the creation of collapsible regions, turning this

```text
``collapse:Title
Content
``
```

into this 

```text
<details>
<summary>Title</summary>
<div class='collapsed'>
Content
</div>
</details>
```
