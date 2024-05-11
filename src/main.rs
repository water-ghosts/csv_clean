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

    title_string
}

fn main() {
    println!("Hello, world!");
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
    let input = " TEST   今日の一枚";
    let expected = "Test 今日の一枚";
    let title = to_title(input);
    assert_eq!(&title, expected);
}
