use super::{pick_port, start_server};
use crate::{
    ActsChannel, ActsOptions, Vars,
    model::{ModelInfo, Package, PageData},
    tests::SERVER_ADDR,
};
use acts::Signal;
use tokio::sync::oneshot;

#[tokio::test]
async fn workflow_deploy() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let yml = r"
    id: test
    name: model test
    ver: 0.1.0
    steps:
        - name: step 1
    ";
    let ret = client.deploy(yml, Some("1")).await.unwrap();

    assert!(ret.data.unwrap());
    tx.send(()).unwrap();
}

#[tokio::test]
async fn workflow_publish() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();
    let yml = r#"
    id: test
    name: test
    desc: test desc
    icon: my-icon
    doc: test doc
    schema:
        test: object
    resources: []
    catalog: app
    run_as: irq
    version: 0.1.0
    "#;

    let package = serde_yaml::from_str::<Package>(yml).unwrap();
    let ret = client.publish(&package).await.unwrap();

    assert!(ret.data.unwrap());
    tx.send(()).unwrap();
}

#[tokio::test]
async fn workflow_start() {
    let (tx, rx) = oneshot::channel();
    let port = pick_port();
    let url = format!("http://{}:{port}", SERVER_ADDR);
    start_server(port, rx).await;

    let mut client = ActsChannel::connect(&url).await.unwrap();

    let value = Vars::new().with("pid", "123");
    client
        .deploy(
            r"
        id: test
        ver: 0.1.0
    ",
            Some("1"),
        )
        .await
        .unwrap();

    let resp = client.start("1", value).await.unwrap();
    assert_eq!(resp.data.unwrap(), "123");
    tx.send(()).unwrap();
}

#[tokio::test]
async fn workflow_subscribe() {
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
    client
        .subscribe(
            "client-1",
            move |m: &crate::model::Message| {
                if m.is_state("created") && m.is_params_key("abc") {
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
async fn workflow_models() {
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
    client.deploy(model, None).await.unwrap();
    let resp = client
        .send::<PageData<ModelInfo>>("model:ls", Vars::new())
        .await
        .unwrap();
    assert_eq!(resp.data.unwrap().count, 1);
    tx.send(()).unwrap();
}
