use std::env;
use std::fs;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

const FIELD_SEP: char = '\x1f';
const QUOTE: char = '"';
const BOM: &str = "\u{feff}";

type ParsedRecord = String;

fn to_title(input: &str) -> String {
    let mut title_string = String::with_capacity(input.len());
    let mut is_after_space = true;

    let input = if input.starts_with(BOM) {
        &input[BOM.len()..]
    } else {
        input
    };

    for next_char in input.chars() {
        let char = if next_char == '_' { ' ' } else { next_char };

        if char == '\n' || (is_after_space && char == ' ') {
            continue;
        }

        if is_after_space {
            // Uppercase it!
            if char.is_lowercase() {
                for upper_char in char.to_uppercase() {
                    title_string.push(upper_char);
                }
            } else {
                title_string.push(char);
            }
        } else {
            // Lowercase it!
            if char.is_uppercase() {
                for lower_char in char.to_lowercase() {
                    title_string.push(lower_char);
                }
            } else {
                title_string.push(char);
            }
        }
        is_after_space = char == ' ';
    }

    // Trim trailing whitespace
    while title_string.ends_with(' ') {
        title_string.pop();
    }

    title_string
}

fn restore_record(raw: &ParsedRecord, sep: char) -> String {
    let mut output = String::with_capacity(raw.len());

    let mut field = String::with_capacity(raw.len());
    let mut needs_quote = false;
    let mut is_first = true;

    fn commit(
        output: &mut String,
        field: &str,
        sep: char,
        is_first_field: bool,
        requires_quote: bool,
    ) {
        if field.is_empty() {
            return;
        }

        if !is_first_field {
            output.push(sep);
        }

        if requires_quote {
            output.push(QUOTE);
        }

        output.push_str(&field);

        if requires_quote {
            output.push(QUOTE);
        }
    }

    for char in raw.chars() {
        if char == FIELD_SEP {
            // Wrap up and commit current field
            commit(&mut output, &field, sep, is_first, needs_quote);

            field.clear();
            needs_quote = false;
            is_first = false;
        } else {
            if char == QUOTE {
                needs_quote = true;
                field.push(char);
            } else if char == sep {
                needs_quote = true;
            }
            field.push(char)
        }
    }

    commit(&mut output, &field, sep, is_first, needs_quote);

    output
}

struct FieldIterator {
    record: Vec<char>,
    pos: usize,
    sep: char,
}

impl FieldIterator {
    fn new(raw: &str, sep: char) -> Self {
        FieldIterator {
            record: raw.chars().collect(),
            pos: 0,
            sep,
        }
    }
}

impl Iterator for FieldIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let chars = &self.record[self.pos..];

        if chars.is_empty() {
            return None;
        }

        let mut parsed = ParsedRecord::new();

        let mut is_quoted = false;
        let mut skip = false;
        let mut is_field_complete = false;

        while self.pos < self.record.len() && !is_field_complete {
            let chars = &self.record[self.pos..];
            let char = chars[0];

            if skip {
                skip = false;
            } else if char == self.sep {
                if is_quoted {
                    parsed.push(char);
                } else {
                    is_field_complete = true;
                }
            } else if char == QUOTE {
                let is_next_char_quote = chars.get(1) == Some(&QUOTE);

                if is_next_char_quote {
                    if !is_quoted && chars.get(2) == Some(&QUOTE) {
                        skip = false;
                        is_quoted = true;
                    } else {
                        skip = true;
                        if is_quoted {
                            parsed.push(QUOTE);
                        }
                    }
                } else {
                    is_quoted = !is_quoted;
                }
            } else {
                parsed.push(char);
            }

            self.pos += 1;
        }

        Some(parsed)
    }
}

fn join(fields: impl Iterator<Item = String>, sep: char) -> String {
    let mut is_first = true;
    let mut output = String::new();

    for field in fields {
        if !is_first {
            output.push(sep);
        }
        output.push_str(&field);
        is_first = false;
    }

    output
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = match args.get(1) {
        Some(path) => path,
        None => "-",
    };

    let mut reader: Box<dyn BufRead> = match filepath {
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(BufReader::new(File::open(filepath).unwrap())),
    };

    let mut header = String::new();
    let _ = reader.read_line(&mut header);

    let separator = if header.contains('\t') { '\t' } else { ',' };
    let output_separator = ';';

    let header_fields = FieldIterator::new(&header, separator).map(|field| to_title(&field));
    let parsed_header = join(header_fields, output_separator);

    let mut parsed_rows: Vec<String> = reader
        .lines()
        .filter_map(|row| row.ok())
        .map(|row| join(FieldIterator::new(&row, separator), FIELD_SEP))
        .collect();

    parsed_rows.sort_unstable();

    // Write BOM (maybe gate this behind a flag?)
    // Write header
    println!("{}{}", BOM, parsed_header);

    // For each sorted row, reconstruct and print
    for row in parsed_rows
        .iter()
        .map(|row| restore_record(row, output_separator))
    {
        println!("{}", row);
    }
}

#[cfg(test)]

fn parse_record(raw: &str, sep: char) -> ParsedRecord {
    join(FieldIterator::new(raw, sep), FIELD_SEP)
}

#[test]
fn test_ascii_to_title() {
    let input = "TOTAL__SPEND";
    let expected = "Total Spend";
    let title = to_title(input);
    assert_eq!(&title, expected);
}

#[test]
fn test_unicode_to_title() {
    let input = " TEST   今日の一枚 test  ";
    let expected = "Test 今日の一枚 Test";
    let title = to_title(input);
    assert_eq!(&title, expected);
}

#[test]
fn test_parse_record() {
    let input = "abc,def,ghi";
    let expected = "abc\x1Fdef\x1Fghi";
    let output = parse_record(input, ',');
    assert_eq!(&output, expected);
}

#[test]
fn test_parse_record_quotes() {
    let input = "abc,\"d,e,f\",jhi";
    let expected = "abc\x1Fd,e,f\x1Fjhi";
    let output = parse_record(input, ',');
    assert_eq!(&output, expected);
}

#[test]
fn test_parse_record_empty_quotes() {
    let input = "x,\"\",z";
    let expected = "x\x1F\x1Fz";
    let output = parse_record(input, ',');
    assert_eq!(&output, expected);
}

#[test]
fn test_parse_record_escaped_quotes() {
    let input = "x,\"\"\"\",z";
    let expected = "x\x1F\"\x1Fz";
    let output = parse_record(input, ',');
    assert_eq!(&output, expected);
}

#[test]
fn parse_and_restore() {
    let input = "abc,def,ghi";
    let expected = "abc;def;ghi";
    let output = restore_record(&parse_record(input, ','), ';');

    assert_eq!(&output, expected);
}

#[test]
fn parse_and_restore_quotes() {
    let input = "abc,\"d,e,f\",ghi";
    let expected = "abc,\"d,e,f\",ghi";
    let output = restore_record(&parse_record(input, ','), ',');

    assert_eq!(&output, expected);
}

#[test]
fn parse_and_restore_no_quotes() {
    let input = "abc,\"d,e,f\",ghi";
    let expected = "abc;d,e,f;ghi";
    let output = restore_record(&parse_record(input, ','), ';');

    assert_eq!(&output, expected);
}
