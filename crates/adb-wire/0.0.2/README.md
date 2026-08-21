# adb-wire

Async ADB wire protocol client over TCP. No spawning `adb` subprocesses.

```rust
use adb_wire::AdbWire;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let adb = AdbWire::new("emulator-5554");

    // Run a command — returns stdout, stderr, and exit code
    let result = adb.shell("getprop ro.build.version.sdk").await.unwrap();
    println!("SDK: {}", result.stdout_str());

    if !result.success() {
        eprintln!("exit {}: {}", result.exit_code, result.stderr_str());
    }

    // fire-and-forget
    adb.shell_detach("input tap 500 500").await.unwrap();

    // stream output (e.g. logcat)
    use tokio::io::{AsyncBufReadExt, BufReader};
    let stream = adb.shell_stream("logcat -d").await.unwrap();
    let mut lines = BufReader::new(stream).lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        println!("{line}");
    }

    // file transfer
    adb.push_file("local.txt", "/sdcard/remote.txt").await.unwrap();
    adb.pull_file("/sdcard/remote.txt", "downloaded.txt").await.unwrap();

    // file stat
    let st = adb.stat("/sdcard/remote.txt").await.unwrap();
    if st.exists() {
        println!("size: {} bytes", st.size);
    }

    // install APK
    adb.install("app.apk").await.unwrap();

    // port forwarding
    adb.forward("tcp:8080", "tcp:8080").await.unwrap();
    adb.reverse("tcp:3000", "tcp:3000").await.unwrap();
}
```

List devices without a serial:

```rust
use adb_wire::{list_devices, DEFAULT_SERVER};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    for (serial, state) in list_devices(DEFAULT_SERVER).await.unwrap() {
        println!("{serial}\t{state}");
    }
}
```

Requires a running ADB server (`adb start-server`).
