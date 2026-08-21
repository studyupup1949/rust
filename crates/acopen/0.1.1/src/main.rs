use clap::Parser;
use std::env::current_dir;
use webbrowser::open;

#[derive(Parser)]
struct Cli {
    problem_num: String,

    #[clap(short, long)]
    submissions: bool,
}

fn main() {
    let args = Cli::parse();
    let problem_num: String = args.problem_num;
    let mut problem_id = problem_num.clone();
    if let Ok(n) = problem_id.parse::<usize>() {
        match number_to_alphabet(n) {
            Ok(s) => problem_id = s,
            Err(err) => println!("{}", err),
        }
    }
    let contest_id = current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        .split('/')
        .last()
        .unwrap()
        .to_string();
    let url = if args.submissions {
        format!(
            "https://atcoder.jp/contests/{}/submissions?f.LanguageName=Rust&f.Status=AC&f.Task={}_{}&f.User=&orderBy=source_length",
            contest_id, contest_id, problem_id
        )
    } else {
        format!(
            "https://atcoder.jp/contests/{}/tasks/{}_{}",
            contest_id, contest_id, problem_id
        )
    };
    if open(&url).is_ok() {
        println!(
            "Open the page with the following problem in your default browser.\nContest: {}\nProblem: {}",
            contest_id, problem_num
        );
    }
}

fn number_to_alphabet(num: usize) -> Result<String, String> {
    let alphabets = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<char>>();
    if num == 0 {
        Err("Invalid argument (0)".to_string())
    } else if num <= 26 {
        Ok(alphabets[num - 1].to_string())
    } else if num <= 26 * 26 {
        Ok(format!(
            "{}{}",
            alphabets[(num - 1) / 26 - 1],
            alphabets[(num - 1) % 26]
        ))
    } else {
        Err("Problem ids greater than 26^2 are not supported.".to_string())
    }
}

#[test]
fn test_number_to_alphabet() {
    assert_eq!(number_to_alphabet(1).unwrap(), "a");
    assert_eq!(number_to_alphabet(52).unwrap(), "az");
    assert_eq!(number_to_alphabet(53).unwrap(), "ba");
    assert_eq!(number_to_alphabet(90).unwrap(), "cl");
    assert_eq!(number_to_alphabet(0).unwrap_err(), "Invalid argument (0)");
    assert_eq!(
        number_to_alphabet(677).unwrap_err(),
        "Problem ids greater than 26^2 are not supported."
    )
}
