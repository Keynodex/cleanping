//! What a terminal would show for some output, line by line: a carriage return goes back to
//! the start of the line and later text overwrites earlier text, as on a real screen.

/// The visible lines of `plain` (output with escape sequences already removed), each without
/// trailing blanks.
pub fn shown_lines(plain: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line: Vec<char> = Vec::new();
    let mut column = 0;
    for c in plain.chars() {
        match c {
            '\r' => column = 0,
            '\n' => {
                lines.push(finish(&line));
                line.clear();
                column = 0;
            }
            _ => {
                if column < line.len() {
                    line[column] = c;
                } else {
                    line.push(c);
                }
                column += 1;
            }
        }
    }
    lines.push(finish(&line));
    lines
}

fn finish(line: &[char]) -> String {
    line.iter().collect::<String>().trim_end().to_string()
}
