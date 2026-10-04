const TABLE: &[(char, &str)] = &[
    ('А', "A"),
    ('а', "a"),
    ('Б', "B"),
    ('б', "b"),
    ('В', "V"),
    ('в', "v"),
    ('Г', "G"),
    ('г', "g"),
    ('Д', "D"),
    ('д', "d"),
    ('Е', "E"),
    ('е', "e"),
    ('Ё', "Yo"),
    ('ё', "yo"),
    ('Ж', "Zh"),
    ('ж', "zh"),
    ('З', "Z"),
    ('з', "z"),
    ('И', "I"),
    ('и', "i"),
    ('Й', "Y"),
    ('й', "y"),
    ('К', "K"),
    ('к', "k"),
    ('Л', "L"),
    ('л', "l"),
    ('М', "M"),
    ('м', "m"),
    ('Н', "N"),
    ('н', "n"),
    ('О', "O"),
    ('о', "o"),
    ('П', "P"),
    ('п', "p"),
    ('Р', "R"),
    ('р', "r"),
    ('С', "S"),
    ('с', "s"),
    ('Т', "T"),
    ('т', "t"),
    ('У', "U"),
    ('у', "u"),
    ('Ф', "F"),
    ('ф', "f"),
    ('Х', "Kh"),
    ('х', "kh"),
    ('Ц', "Ts"),
    ('ц', "ts"),
    ('Ч', "Ch"),
    ('ч', "ch"),
    ('Ш', "Sh"),
    ('ш', "sh"),
    ('Щ', "Sch"),
    ('щ', "sch"),
    ('Ъ', "'"),
    ('ъ', "'"),
    ('Ы', "Y"),
    ('ы', "y"),
    ('Ь', "'"),
    ('ь', "'"),
    ('Э', "E"),
    ('э', "e"),
    ('Ю', "Yu"),
    ('ю', "yu"),
    ('Я', "Ya"),
    ('я', "ya"),
];

pub fn transliterate(s: &str) -> String {
    s.chars()
        .map(|c| match TABLE.iter().find(|(from, _)| *from == c) {
            Some((_, to)) => to.to_string(),
            None => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translit() {
        assert_eq!(transliterate("Азатьян"), "Azat'yan");
        assert_eq!(transliterate("Георгий"), "Georgiy");
        assert_eq!(transliterate("Щёлоков Юрий"), "Schyolokov Yuriy");
        assert_eq!(transliterate("abc"), "abc");
    }
}
