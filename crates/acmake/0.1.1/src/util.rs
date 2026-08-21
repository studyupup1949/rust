pub fn get_current_path() -> std::io::Result<String> {
    std::env::current_dir()
        .and_then(|p| {
            p.into_os_string()
                .into_string()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert OsString to String"))
        })
}

pub fn create_dir<T1, T2>(parent: T1, name: T2) -> std::io::Result<String>
where
    T1: AsRef<str>,
    T2: AsRef<str>,
{
    let path = format!("{}/{}", parent.as_ref(), name.as_ref());
    std::fs::create_dir(&path)?;

    Ok(path)
}
