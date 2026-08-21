
const COLOR_START: &str = "\x1B[";
const COLOR_RESET: &str = "\x1B[0m";

fn color(color: &str, message: &str) -> String {
    format!("{}{}m{}{}", COLOR_START, color, message, COLOR_RESET)
}

fn main() {
    println!("{} is everyone's favourite cli tool: yet another dotfiles manager \\o/", color("32;1", "adhd"));
    println!("{}", color("2", "\n\t(And it's a work in progress. Because adhd)\n"));
}
