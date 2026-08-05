use crate::Tendril;

// todo: should this be grapheme aware?

fn split_case_words(text: impl Iterator<Item = char>) -> Vec<Tendril> {
    let mut words = Vec::new();
    let mut current = Tendril::new();
    let mut chars = text.peekable();
    let mut previous: Option<char> = None;

    while let Some(c) = chars.next() {
        if !c.is_alphanumeric() {
            if !current.is_empty() {
                words.push(current);
                current = Tendril::new();
            }
            previous = None;
            continue;
        }

        let boundary = match previous {
            Some(prev)
                if (prev.is_lowercase() || prev.is_numeric())
                    && c.is_uppercase()
                    && matches!(chars.peek(), Some(next) if next.is_lowercase()) =>
            {
                true
            }
            Some(prev)
                if prev.is_numeric()
                    && c.is_alphabetic()
                    && (!c.is_uppercase()
                        || matches!(chars.peek(), Some(next) if next.is_lowercase())) =>
            {
                true
            }
            Some(prev)
                if prev.is_uppercase()
                    && c.is_uppercase()
                    && matches!(chars.peek(), Some(next) if next.is_lowercase()) =>
            {
                true
            }
            _ => false,
        };

        if boundary && !current.is_empty() {
            words.push(current);
            current = Tendril::new();
        }

        current.push(c);
        previous = Some(c);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn push_title_word(word: &Tendril, buf: &mut Tendril) {
    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        buf.extend(first.to_uppercase());
        for c in chars {
            buf.extend(c.to_lowercase());
        }
    }
}

fn push_camel_word(word: &Tendril, is_first_word: bool, buf: &mut Tendril) {
    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        if is_first_word {
            buf.extend(first.to_lowercase());
        } else {
            buf.extend(first.to_uppercase());
        }
        for c in chars {
            buf.extend(c.to_lowercase());
        }
    }
}

pub fn to_pascal_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_pascal_case_with(text, &mut res);
    res
}

pub fn to_pascal_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for word in split_case_words(text) {
        push_title_word(&word, buf);
    }
}

pub fn to_upper_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for c in text {
        for c in c.to_uppercase() {
            buf.push(c)
        }
    }
}

pub fn to_lower_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for c in text {
        for c in c.to_lowercase() {
            buf.push(c)
        }
    }
}

pub fn to_camel_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_camel_case_with(text, &mut res);
    res
}
pub fn to_camel_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for (index, word) in split_case_words(text).iter().enumerate() {
        push_camel_word(word, index == 0, buf);
    }
}

pub fn to_snake_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_snake_case_with(text, &mut res);
    res
}
pub fn to_snake_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for (index, word) in split_case_words(text).iter().enumerate() {
        if index > 0 {
            buf.push('_');
        }
        for c in word.chars() {
            buf.extend(c.to_lowercase());
        }
    }
}

pub fn to_kebab_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_kebab_case_with(text, &mut res);
    res
}
pub fn to_kebab_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for (index, word) in split_case_words(text).iter().enumerate() {
        if index > 0 {
            buf.push('-');
        }
        for c in word.chars() {
            buf.extend(c.to_lowercase());
        }
    }
}

pub fn to_title_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_title_case_with(text, &mut res);
    res
}
pub fn to_title_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    let mut at_word_start = true;

    for c in text {
        if !c.is_alphanumeric() {
            at_word_start = true;
            buf.extend(c.to_lowercase());
        } else if at_word_start {
            at_word_start = false;
            buf.extend(c.to_uppercase());
        } else {
            buf.extend(c.to_lowercase());
        }
    }
}

pub fn to_sentence_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_sentence_case_with(text, &mut res);
    res
}
pub fn to_sentence_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    let mut at_sentence_start = true;

    for c in text {
        if c == '.' || c == '?' || c == '!' {
            at_sentence_start = true;
            buf.push(c);
        } else if at_sentence_start {
            if c.is_alphabetic() {
                at_sentence_start = false;
                buf.extend(c.to_uppercase());
            } else {
                buf.extend(c.to_lowercase());
            }
        } else {
            buf.extend(c.to_lowercase());
        }
    }
}

pub fn to_alternate_case(text: impl Iterator<Item = char>) -> Tendril {
    let mut res = Tendril::new();
    to_alternate_case_with(text, &mut res);
    res
}
pub fn to_alternate_case_with(text: impl Iterator<Item = char>, buf: &mut Tendril) {
    for c in text {
        if c.is_lowercase() {
            buf.extend(c.to_uppercase());
        } else if c.is_uppercase() {
            buf.extend(c.to_lowercase());
        } else {
            buf.push(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_underscore() {
        let result = to_camel_case("otto_botto".chars());
        assert_eq!(result.as_ref() as &str, "ottoBotto");
    }

    #[test]
    fn test_camel_case_uppercase() {
        let result = to_camel_case("OTTO_BOTTO".chars());
        assert_eq!(result.as_ref() as &str, "ottoBotto");
    }

    #[test]
    fn test_camel_case_mixed_case() {
        let result = to_camel_case("OttO_boTTO".chars());
        assert_eq!(result.as_ref() as &str, "ottoBotto");
    }

    #[test]
    fn test_camel_case_includes_nums() {
        let result = to_camel_case("Ott0_b0TT0".chars());
        assert_eq!(result.as_ref() as &str, "ott0B0tt0");
    }

    #[test]
    fn test_camel_case_one_word_lower() {
        let result = to_camel_case("otto".chars());
        assert_eq!(result.as_ref() as &str, "otto");
    }

    #[test]
    fn test_camel_case_one_word_upper() {
        let result = to_camel_case("OTTO".chars());
        assert_eq!(result.as_ref() as &str, "otto");
    }

    #[test]
    fn test_camel_case_one_char_lower() {
        let result = to_camel_case("o".chars());
        assert_eq!(result.as_ref() as &str, "o");
    }

    #[test]
    fn test_camel_case_one_char_upper() {
        let result = to_camel_case("O".chars());
        assert_eq!(result.as_ref() as &str, "o");
    }

    #[test]
    fn test_camel_case_pascal_input() {
        let result = to_camel_case("PascalCase".chars());
        assert_eq!(result.as_ref() as &str, "pascalCase");
    }

    #[test]
    fn test_camel_case_many_words_separators() {
        let result = to_camel_case("otto_botto_the_dog".chars());
        assert_eq!(result.as_ref() as &str, "ottoBottoTheDog");
    }

    #[test]
    fn test_camel_case_empty_string() {
        let result = to_camel_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_snake_case_simple() {
        let result = to_snake_case("helloWorld".chars());
        assert_eq!(result.as_ref() as &str, "hello_world");
    }

    #[test]
    fn test_snake_case_multiple_words() {
        let result = to_snake_case("helloWorldTest".chars());
        assert_eq!(result.as_ref() as &str, "hello_world_test");
    }

    #[test]
    fn test_snake_case_with_spaces() {
        let result = to_snake_case("hello world test".chars());
        assert_eq!(result.as_ref() as &str, "hello_world_test");
    }

    #[test]
    fn test_snake_case_with_underscores() {
        let result = to_snake_case("hello_world_test".chars());
        assert_eq!(result.as_ref() as &str, "hello_world_test");
    }

    #[test]
    fn test_snake_case_all_upper() {
        let result = to_snake_case("HELLO_WORLD".chars());
        assert_eq!(result.as_ref() as &str, "hello_world");
    }

    #[test]
    fn test_snake_case_pascal_input() {
        let result = to_snake_case("PascalCase".chars());
        assert_eq!(result.as_ref() as &str, "pascal_case");
    }

    #[test]
    fn test_snake_case_camel_input() {
        let result = to_snake_case("camelCase".chars());
        assert_eq!(result.as_ref() as &str, "camel_case");
    }

    #[test]
    fn test_snake_case_empty() {
        let result = to_snake_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_snake_case_numbers() {
        let result = to_snake_case("test123case".chars());
        assert_eq!(result.as_ref() as &str, "test123_case");
    }

    #[test]
    fn test_kebab_case_simple() {
        let result = to_kebab_case("helloWorld".chars());
        assert_eq!(result.as_ref() as &str, "hello-world");
    }

    #[test]
    fn test_kebab_case_multiple_words() {
        let result = to_kebab_case("helloWorldTest".chars());
        assert_eq!(result.as_ref() as &str, "hello-world-test");
    }

    #[test]
    fn test_kebab_case_with_spaces() {
        let result = to_kebab_case("hello world test".chars());
        assert_eq!(result.as_ref() as &str, "hello-world-test");
    }

    #[test]
    fn test_kebab_case_all_upper() {
        let result = to_kebab_case("HELLO_WORLD".chars());
        assert_eq!(result.as_ref() as &str, "hello-world");
    }

    #[test]
    fn test_kebab_case_pascal_input() {
        let result = to_kebab_case("PascalCase".chars());
        assert_eq!(result.as_ref() as &str, "pascal-case");
    }

    #[test]
    fn test_kebab_case_camel_input() {
        let result = to_kebab_case("camelCase".chars());
        assert_eq!(result.as_ref() as &str, "camel-case");
    }

    #[test]
    fn test_kebab_case_empty() {
        let result = to_kebab_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_pascal_case_from_camel_and_snake() {
        assert_eq!(to_pascal_case("camelCase".chars()).as_ref() as &str, "CamelCase");
        assert_eq!(to_pascal_case("snake_case".chars()).as_ref() as &str, "SnakeCase");
    }

    #[test]
    fn test_title_case_simple() {
        let result = to_title_case("hello world".chars());
        assert_eq!(result.as_ref() as &str, "Hello World");
    }

    #[test]
    fn test_title_case_all_upper() {
        let result = to_title_case("HELLO WORLD".chars());
        assert_eq!(result.as_ref() as &str, "Hello World");
    }

    #[test]
    fn test_title_case_mixed() {
        let result = to_title_case("hELLO wORLD".chars());
        assert_eq!(result.as_ref() as &str, "Hello World");
    }

    #[test]
    fn test_title_case_empty() {
        let result = to_title_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_sentence_case_simple() {
        let result = to_sentence_case("hello world. how are you?".chars());
        assert_eq!(result.as_ref() as &str, "Hello world. How are you?");
    }

    #[test]
    fn test_sentence_case_all_upper() {
        let result = to_sentence_case("HELLO WORLD".chars());
        assert_eq!(result.as_ref() as &str, "Hello world");
    }

    #[test]
    fn test_sentence_case_empty() {
        let result = to_sentence_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_alternate_case_simple() {
        let result = to_alternate_case("Hello World".chars());
        assert_eq!(result.as_ref() as &str, "hELLO wORLD");
    }

    #[test]
    fn test_alternate_case_all_lower() {
        let result = to_alternate_case("hello world".chars());
        assert_eq!(result.as_ref() as &str, "HELLO WORLD");
    }

    #[test]
    fn test_alternate_case_all_upper() {
        let result = to_alternate_case("HELLO WORLD".chars());
        assert_eq!(result.as_ref() as &str, "hello world");
    }

    #[test]
    fn test_alternate_case_empty() {
        let result = to_alternate_case("".chars());
        assert_eq!(result.as_ref() as &str, "");
    }

    #[test]
    fn test_alternate_case_mixed() {
        let result = to_alternate_case("HeLLo WoRLD".chars());
        assert_eq!(result.as_ref() as &str, "hElLo wOrld");
    }
}
