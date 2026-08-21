use crate::{
    ActsChannel, ActsOptions, Vars,
    tests::{SERVER_ADDR, pick_port, start_server},
};
use acts::Signal;
use tokio::sync::oneshot;

#[tokio::test]
async fn act_submit() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: abc
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("abc") {
                    let m = m.clone();
                    let mut c = c.clone();
                    let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                    tokio::spawn(async move {
                        c.send::<()>("act:submit", vars).await.unwrap();
                    });
                }

                if m.is_state("submitted") && m.is_params_key("abc") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_complete() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: abc
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("abc") {
                    let m = m.clone();
                    let mut c = c.clone();
                    let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                    tokio::spawn(async move {
                        c.send::<()>("act:complete", vars).await.unwrap();
                    });
                }

                if m.is_state("completed") && m.is_params_key("abc") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_push() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: abc
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.nid == "step1" {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new()
                            .with("pid", &m.pid)
                            .with("tid", &m.tid)
                            .with("uses", "acts.core.irq")
                            .with("params", Vars::new().with("key", "push_act_key"));
                        c.send::<()>("act:push", vars).await.unwrap();
                    });
                }

                if m.is_state("created") && m.is_params_key("push_act_key") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_remove() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: abc
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("abc") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                        c.send::<()>("act:remove", vars).await.unwrap();
                    });
                }

                if m.is_state("removed") && m.is_params_key("abc") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_error() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: abc
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("abc") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new()
                            .with("pid", &m.pid)
                            .with("tid", &m.tid)
                            .with("ecode", "err1");
                        c.send::<()>("act:error", vars).await.unwrap();
                    });
                }

                if m.is_state("error") && m.is_params_key("abc") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_back() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: act1
        - name: step 2
          id: step2
          uses: acts.core.irq
          params:
            key: act2
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("act1") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                        c.send::<()>("act:complete", vars).await.unwrap();
                    });
                }

                if m.is_state("created") && m.is_params_key("act2") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new()
                            .with("pid", &m.pid)
                            .with("tid", &m.tid)
                            .with("to", "step1");
                        c.send::<()>("act:back", vars).await.unwrap();
                    });
                }

                if m.is_state("backed") && m.is_params_key("act2") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_cancel() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: act1
        - name: step 2
          id: step2
          uses: acts.core.irq
          params:
            key: act2
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("act1") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                        c.send::<()>("act:complete", vars).await.unwrap();
                    });
                }

                if m.is_state("completed") && m.is_params_key("act1") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new().with("pid", &m.pid).with("tid", &m.tid);
                        c.send::<()>("act:cancel", vars).await.unwrap();
                    });
                }

                if m.is_state("cancelled") && m.is_params_key("act2") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn act_skip() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let model = r#"
    id: test
    ver: 0.1.0
    steps:
        - name: step 1
          id: step1
          uses: acts.core.irq
          params:
            key: act1
    "#;
    client.deploy(model, Some("1")).await.unwrap();
    let (s, s2) = Signal::new(()).double();
    let c = client.clone();
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("act1") {
                    let m = m.clone();
                    let mut c = c.clone();
                    tokio::spawn(async move {
                        let vars = Vars::new()
                            .with("pid", &m.pid)
                            .with("tid", &m.tid)
                            .with("var1", "value1");
                        c.send::<()>("act:skip", vars).await.unwrap();
                    });
                }

                if m.is_state("skipped") && m.is_params_key("act1") {
                    s.close();
                }
            },
            &ActsOptions::default(),
        )
        .await;
    let resp = client
        .start("1", Vars::new().with("pid", "123"))
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    s2.recv().await;
    tx.send(()).unwrap();
}
