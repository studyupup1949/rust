use qinpel_srv::actix_web;
use qinpel_srv::QinServer;

mod clip;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = clip::parse();
    let arg_debug = if args.is_present("debug") {
        Some(true)
    } else {
        None
    };
    let arg_verbose = if args.is_present("verbose") {
        Some(true)
    } else {
        None
    };
    let arg_host = if args.is_present("host") {
        Some(String::from(
            args.value_of("host")
                .expect("Could not read the host argument."),
        ))
    } else {
        None
    };
    let arg_port: Option<u64> = if args.is_present("port") {
        Some(
            args.value_of("port")
                .expect("Could not read the port argument.")
                .parse()
                .expect("Could not parse the port argument."),
        )
    } else {
        None
    };
    let server = QinServer {
        debug: arg_debug,
        verbose: arg_verbose,
        server_name: Some("Abacuz".into()),
        server_host: arg_host,
        server_port: arg_port,
        serves_pubs: Some(true),
        serves_apps: Some(false),
        serves_dirs: Some(true),
        serves_cmds: Some(false),
        serves_sqls: Some(false),
        serves_lizs: Some(false),
        redirects: None,
    };
    qinpel_srv::start(server).await
}
