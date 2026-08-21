pub fn clear_terminal() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/c", "cls"])
            .status();
    }

    #[cfg(unix)]
    {
        let _ = std::process::Command::new("clear").status();
    }
}