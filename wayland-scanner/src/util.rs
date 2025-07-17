pub fn snake_to_camel<S: AsRef<str>>(s: S) -> String
{
    s.as_ref()
        .split('_')
        .flat_map(|part| {
            let mut first = true;
            part.chars().map(move |c| {
                if first {
                    first = false;
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
        })
        .collect()
}
