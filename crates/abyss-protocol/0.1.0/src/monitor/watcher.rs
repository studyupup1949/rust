use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use rand::Rng;
use walkdir::WalkDir;

use super::parser::{parse_jsonl_line, TokenEvent};

/// 发现所有 JSONL 文件所在的目录路径。
/// 优先使用环境变量 CLAUDE_DATA_PATHS（冒号分隔）或 CLAUDE_DATA_PATH（单路径），
/// 否则扫描默认路径 ~/.claude/projects/ 和 ~/.config/claude/projects/。
pub fn discover_paths() -> Vec<PathBuf> {
    // 环境变量覆盖
    if let Ok(paths) = std::env::var("CLAUDE_DATA_PATHS") {
        let dirs: Vec<PathBuf> = paths
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();
        if !dirs.is_empty() {
            return collect_jsonl_files(&dirs);
        }
    }

    if let Ok(path) = std::env::var("CLAUDE_DATA_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return collect_jsonl_files(&[p]);
        }
    }

    // 默认路径
    let mut base_dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        let claude_dir = home.join(".claude").join("projects");
        if claude_dir.exists() {
            base_dirs.push(claude_dir);
        }
    }
    if let Some(config) = dirs::config_dir() {
        let claude_config = config.join("claude").join("projects");
        if claude_config.exists() {
            base_dirs.push(claude_config);
        }
    }

    collect_jsonl_files(&base_dirs)
}
/// 递归扫描目录列表，收集所有 .jsonl 文件
fn collect_jsonl_files(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in base_dirs {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "jsonl") {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

/// 从发现的 JSONL 文件路径中提取唯一的父目录，用于 notify 监听
fn discover_watch_dirs() -> Vec<PathBuf> {
    let files = discover_paths();
    let mut dirs: HashSet<PathBuf> = HashSet::new();
    for f in &files {
        if let Some(parent) = f.parent() {
            dirs.insert(parent.to_path_buf());
        }
    }

    // 如果没有发现任何文件，也把默认目录加进来以便监听新文件
    if dirs.is_empty() {
        if let Some(home) = dirs::home_dir() {
            let claude_dir = home.join(".claude").join("projects");
            if claude_dir.exists() {
                dirs.insert(claude_dir);
            }
        }
        if let Some(config) = dirs::config_dir() {
            let claude_config = config.join("claude").join("projects");
            if claude_config.exists() {
                dirs.insert(claude_config);
            }
        }
    }

    dirs.into_iter().collect()
}

/// JSONL 文件监控器，监听文件变更并解析 TokenEvent
pub struct TokenMonitor {
    _watcher: Option<RecommendedWatcher>,
    event_rx: std_mpsc::Receiver<notify::Result<Event>>,
    file_offsets: HashMap<PathBuf, u64>,
    seen_keys: HashSet<(String, String)>,
    tx: tokio::sync::mpsc::Sender<TokenEvent>,
    rx: tokio::sync::mpsc::Receiver<TokenEvent>,
}

impl TokenMonitor {
    /// 创建监控器，自动发现 JSONL 路径并开始监听
    pub fn new() -> Result<Self> {
        let (notify_tx, notify_rx) = std_mpsc::channel();
        let (tx, rx) = tokio::sync::mpsc::channel::<TokenEvent>(256);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = notify_tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        // 监听发现的目录
        let watch_dirs = discover_watch_dirs();
        for dir in &watch_dirs {
            let _ = watcher.watch(dir, RecursiveMode::Recursive);
        }

        // 初始化文件 offset：跳到文件末尾，只处理新增内容
        let mut file_offsets = HashMap::new();
        let files = discover_paths();
        for f in files {
            if let Ok(meta) = std::fs::metadata(&f) {
                file_offsets.insert(f, meta.len());
            }
        }

        Ok(Self {
            _watcher: Some(watcher),
            event_rx: notify_rx,
            file_offsets,
            seen_keys: HashSet::new(),
            tx,
            rx,
        })
    }

    /// 非阻塞地获取所有待处理的 token 事件。
    /// 先处理 notify 文件变更事件，再从 channel 收集结果。
    pub fn poll_events(&mut self) -> Vec<TokenEvent> {
        // 处理所有待处理的 notify 事件
        while let Ok(Ok(event)) = self.event_rx.try_recv() {
            for path in event.paths {
                if path.extension().map_or(false, |ext| ext == "jsonl") {
                    self.read_new_lines(&path);
                }
            }
        }

        // 收集 channel 中的所有事件
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// 从文件的上次 offset 位置读取新行并解析
    fn read_new_lines(&mut self, path: &PathBuf) {
        let offset = self.file_offsets.get(path).copied().unwrap_or(0);

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut reader = BufReader::new(file);
        if reader.seek(SeekFrom::Start(offset)).is_err() {
            return;
        }

        let mut new_offset = offset;
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    new_offset += n as u64;
                    if let Some(event) = parse_jsonl_line(&line) {
                        let key = event.dedup_key();
                        if self.seen_keys.insert(key) {
                            let _ = self.tx.try_send(event);
                        }
                    }
                }
                Err(_) => break,
            }
        }

        self.file_offsets.insert(path.clone(), new_offset);
    }
}

/// Mock 监控器，用于无 Claude Code 环境下的开发测试。
/// 每 2-5 秒随机生成模拟 TokenEvent。
pub struct MockMonitor {
    rx: tokio::sync::mpsc::Receiver<TokenEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl MockMonitor {
    /// 启动 mock 监控器，在后台 task 中定时生成模拟事件
    pub fn start() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<TokenEvent>(256);

        let handle = tokio::spawn(async move {
            let mut counter: u64 = 0;
            loop {
                let delay_ms = {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(2000..=5000)
                };
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                let event = generate_mock_event(&mut counter);
                if tx.send(event).await.is_err() {
                    break; // receiver dropped
                }
            }
        });

        Self {
            rx,
            _handle: handle,
        }
    }

    /// 非阻塞地获取所有待处理的模拟 token 事件
    pub fn poll_events(&mut self) -> Vec<TokenEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }
}

/// 生成一个随机的模拟 TokenEvent
fn generate_mock_event(counter: &mut u64) -> TokenEvent {
    let mut rng = rand::thread_rng();
    *counter += 1;

    TokenEvent {
        timestamp: chrono::Utc::now(),
        message_id: format!("mock_msg_{}", counter),
        request_id: format!("mock_req_{}", counter),
        model: "claude-sonnet-4-20250514".to_string(),
        input_tokens: rng.gen_range(100..=2000),
        output_tokens: rng.gen_range(50..=500),
        cache_creation_tokens: rng.gen_range(0..=200),
        cache_read_tokens: rng.gen_range(0..=500),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_paths_returns_vec() {
        // discover_paths 应该总是返回一个 Vec（可能为空）
        // 不设置环境变量，使用默认路径逻辑
        let paths = discover_paths();
        // 所有返回的路径都应该是 .jsonl 文件
        for p in &paths {
            assert!(p.extension().map_or(false, |ext| ext == "jsonl"));
        }
    }

    #[test]
    fn test_collect_jsonl_files_empty_dirs() {
        let files = collect_jsonl_files(&[]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_generate_mock_event() {
        let mut counter = 0u64;
        let event = generate_mock_event(&mut counter);
        assert_eq!(counter, 1);
        assert_eq!(event.message_id, "mock_msg_1");
        assert_eq!(event.request_id, "mock_req_1");
        assert!(event.input_tokens >= 100 && event.input_tokens <= 2000);
        assert!(event.output_tokens >= 50 && event.output_tokens <= 500);
    }
}
