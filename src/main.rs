use std::process::Output;

const FIELD_SEP: char = '\x1F';
const QUOTE: char = '"';

type ParsedRecord = String;

fn to_title(input: &str) -> String {
    let mut title_string = String::with_capacity(input.len());
    let mut is_after_space = true;

    for next_char in input.chars() {
        let char = if next_char == '_' { ' ' } else { next_char };

        if is_after_space && char == ' ' {
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

fn parse_record(raw: &str, sep: char) -> ParsedRecord {
    let mut parsed = ParsedRecord::with_capacity(raw.len());

    // let mut prev_char = '\0';
    let mut is_quoted = false;
    let mut skip = false;

    for (i, char) in raw.char_indices() {
        println!("{} {} {} {} - {}", i, char, skip, is_quoted, &parsed);
        if skip {
            skip = false;
            continue;
        }

        if char == sep {
            if is_quoted {
                parsed.push(char);
            } else {
                parsed.push(FIELD_SEP);
            }
        } else if char == QUOTE {
            let next_char = raw.chars().nth(i + 1).unwrap_or('\0');
            // Turn double quotes into single quotes within quotes

            if next_char == QUOTE {
                let next_next_char = raw.chars().nth(i + 2).unwrap_or('\0');
                if next_next_char == QUOTE && !is_quoted {
                    is_quoted = true;
                    skip = false;
                } else {
                    skip = true;
                    if is_quoted {
                        parsed.push(QUOTE);
                    }
                }
            } else {
                is_quoted = !is_quoted;
            }

            // Case 1: x,"y",z
            // Case 2: x,"",z becomes x,,z
            // Case 3: x,"""",z becomes a single quote mark
        } else {
            parsed.push(char);
        }
    }
    parsed
}

fn main() {
    let input = "x,\"\"\"\",z";
    let output = parse_record(input, ',');
    println!("{}", output)
    // Write BOM
    // Convert header to title case

    // Infer separator

    //For each row:
    // Parse it to be SEP-delimited (no quotes or anything)
    // Add it to a list

    // Sort the list of rows

    //For each sorted row:
    // Convert the SEP-delimited row to use the canonical separator and quotes
}

#[cfg(test)]
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
