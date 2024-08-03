use std::{fs::File, io::Read, path::PathBuf};

pub fn text_language(max_width: u16, lang: &str) -> Result<Vec<char>, std::io::Error> {
    let mut file = {
        let mut path = ["languages", lang].iter().collect::<PathBuf>();
        path.set_extension("txt");
        File::open(path)?
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let words = buf
        .split_ascii_whitespace()
        .filter(|s| s.len() > 1)
        .map(|s| s.to_lowercase().chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut text = vec![];
    for word in words.iter() {
        if text.len() + word.len() < max_width as usize {
            if !text.is_empty() {
                text.push(' ');
            }
            text.extend(word);
        } else {
            break;
        }
    }
    Ok(text)
}
