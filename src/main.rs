use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use serde::Deserialize;
use std::io;
use std::process;

pub fn make_app() -> App<'static, 'static> {
    App::new("mdbook-regex")
        .about("A preprocessor using regex patterns to find and replace expressions in .md files.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = RegexProcessor;
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    }
    if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

#[derive(Deserialize)]
struct RegexPattern {
    pattern: String,
    template: String,
}

struct RegexProcessor;

impl Preprocessor for RegexProcessor {
    fn name(&self) -> &str {
        "regex"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let patterns = self.build_patterns(ctx);
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                for (pattern, template) in &patterns {
                    chapter.content = self.process_chapter(&chapter.content, &pattern, &template)
                }
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

impl RegexProcessor {
    fn build_patterns(&self, ctx: &PreprocessorContext) -> Vec<(Regex, String)> {
        let config = ctx.config.get_preprocessor("regex").unwrap();
        let patterns_path = match config.get("patterns").unwrap() {
            toml::value::Value::String(macros_value) => Path::new(macros_value),
            _ => panic!("no pattern file supplied"),
        };
        self.load_patterns(patterns_path)
    }

    fn load_patterns(&self, pattern_path: &Path) -> Vec<(Regex, String)> {
        let mut result = Vec::new();
        let text_file = load_as_string(pattern_path);
        let pattern_list: Vec<RegexPattern> = serde_json::from_str(&text_file).unwrap();
        for pattern in pattern_list {
            let regex_pattern = Regex::new(&pattern.pattern).unwrap();
            result.push((regex_pattern, pattern.template));
        }
        result
    }

    fn process_chapter(&self, content: &str, pattern: &Regex, template: &str) -> String {
        let result = pattern.replace_all(content, template);
        String::from(result)
    }
}

fn load_as_string(path: &Path) -> String {
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => (),
    };
    string
}
