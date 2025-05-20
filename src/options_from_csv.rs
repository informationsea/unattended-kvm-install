use anyhow::Context;
use csv;
use std::io::Read;

pub fn generate_options_from_csv(reader: impl Read) -> anyhow::Result<Vec<Vec<String>>> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .has_headers(true)
        .from_reader(reader);
    let header = csv_reader
        .headers()
        .context("Failed to read header")?
        .clone();
    let records = csv_reader.records();
    let mut cmd_list = Vec::new();
    for row in records {
        let mut cmd = Vec::new();
        let record = row?;
        for (header, value) in header.iter().zip(record.iter()) {
            if value != "FALSE" {
                cmd.push(format!("--{}", header));
                if value != "TRUE" {
                    cmd.push(value.to_string());
                }
            }
        }
        cmd_list.push(cmd);
    }
    Ok(cmd_list)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_generate_options_from_csv() {
        let csv_data = r#"foo,bar,hoge
x,y,TRUE
a,b,FALSE
"#;

        let options = generate_options_from_csv(csv_data.as_bytes()).unwrap();
        assert_eq!(
            vec![
                vec!["--foo", "x", "--bar", "y", "--hoge"],
                vec!["--foo", "a", "--bar", "b"],
            ],
            options
        );
    }
}
