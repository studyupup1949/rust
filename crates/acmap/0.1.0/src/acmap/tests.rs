use super::AcMap;

#[tokio::test]
async fn test_insert_get_remove() {
    let amap = AcMap::<u64, u64>::new();

    assert!(amap.is_empty().await);
    assert_eq!(amap.insert(1, 10).await, None);
    assert_eq!(amap.insert(1, 12).await, Some(10));
    assert!(amap.contains_key(1).await);
    assert_eq!(amap.get(1).await, Some(12));
    assert_eq!(amap.len().await, 1);

    assert_eq!(amap.remove(1).await, Some((1, 12)));
    assert_eq!(amap.get(1).await, None);
    assert!(amap.is_empty().await);
}

#[tokio::test]
async fn test_clear() {
    let amap = AcMap::<u64, u64>::with_capacity(16);
    amap.insert(1, 1).await;
    amap.insert(2, 2).await;
    assert_eq!(amap.len().await, 2);

    amap.clear();

    assert_eq!(amap.len().await, 0);
    assert!(amap.is_empty().await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_insert() {
    let amap = AcMap::<u64, u64>::new();
    let workers: u64 = 4;
    let per_worker: u64 = 10_000;

    let mut handles = Vec::new();
    for worker in 0..workers {
        let amap = amap.clone();
        handles.push(tokio::spawn(async move {
            let start = worker * per_worker;
            let end = start + per_worker;
            for i in start..end {
                amap.insert(i, i).await;
            }
        }));
    }

    for handle in handles {
        handle.await.expect("concurrent worker panicked");
    }

    assert_eq!(amap.len().await, (workers * per_worker) as usize);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_insert_fast() {
    let amap = AcMap::<u64, u64>::new();
    let workers: u64 = 4;
    let per_worker: u64 = 10_000;

    let mut handles = Vec::new();
    for worker in 0..workers {
        let amap = amap.clone();
        handles.push(tokio::spawn(async move {
            let start = worker * per_worker;
            let end = start + per_worker;
            for i in start..end {
                amap.insert_fast(i, i);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("concurrent worker panicked");
    }

    assert_eq!(amap.len().await, (workers * per_worker) as usize);
}

#[tokio::test]
async fn test_insert_fast_batch() {
    let amap = AcMap::<u64, u64>::new();
    amap.insert_fast_batch((0..10_000).map(|i| (i, i)));

    assert_eq!(amap.len().await, 10_000);
    assert_eq!(amap.get(7).await, Some(7));
}
