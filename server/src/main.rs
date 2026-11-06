use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc};
use villmarks_server::{router, run_host, App, Host, Msg};

struct Opts {
    port: u16,
    window_secs: f64,
    manual: bool,
    data: PathBuf,
    config: PathBuf,
    static_dir: PathBuf,
    seed: u64,
    snapshot_every: u32,
}

#[tokio::main]
async fn main() {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let host = match Host::open(
        &opts.data,
        &opts.config,
        opts.seed,
        opts.snapshot_every,
        opts.manual,
        opts.window_secs,
    ) {
        Ok(host) => host,
        Err(err) => {
            eprintln!("could not open the valley: {err}");
            std::process::exit(1);
        }
    };
    let (tx, rx) = mpsc::channel(256);
    let (ticks, _) = broadcast::channel(64);
    let shutdown = tx.clone();
    let app = App {
        tx,
        ticks: ticks.clone(),
    };
    let router = router(app, opts.static_dir.clone());
    let host_task = tokio::spawn(run_host(host, rx, ticks));
    let address = format!("127.0.0.1:{}", opts.port);
    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("could not bind {address}: {err}");
            let _ = shutdown.send(Msg::Shutdown).await;
            let _ = host_task.await;
            std::process::exit(1);
        }
    };
    println!("Villmarks valley at http://{address}");
    println!(
        "window {}s, manual {}, data {}",
        opts.window_secs,
        opts.manual,
        opts.data.display()
    );
    if !opts.static_dir.is_dir() {
        println!(
            "web client not built ({}); API is up. In web/: npm install && npm run build",
            opts.static_dir.display()
        );
    }
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = shutdown.send(Msg::Shutdown).await;
        })
        .await
        .ok();
    let _ = host_task.await;
}

fn parse_args() -> Result<Opts, String> {
    let mut port = None;
    let mut window_secs = None;
    let mut manual = false;
    let mut data = None;
    let mut config = PathBuf::from("config/evening.toml");
    let mut static_dir = PathBuf::from("web/dist");
    let mut seed = 1u64;
    let mut snapshot_every = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err(HELP.into()),
            "--manual" => manual = true,
            "--port" => port = Some(parse_next(&mut args, "--port")?),
            "--window-secs" => window_secs = Some(parse_next(&mut args, "--window-secs")?),
            "--data" => data = Some(PathBuf::from(next_string(&mut args, "--data")?)),
            "--config" => config = PathBuf::from(next_string(&mut args, "--config")?),
            "--static" => static_dir = PathBuf::from(next_string(&mut args, "--static")?),
            "--seed" => seed = parse_next(&mut args, "--seed")?,
            "--snapshot-every" => snapshot_every = Some(parse_next(&mut args, "--snapshot-every")?),
            other => return Err(format!("unknown argument {other}\n{HELP}")),
        }
    }
    let file = read_server_defaults(&config);
    Ok(Opts {
        port: port.or(file.port).unwrap_or(18431),
        window_secs: window_secs.or(file.window_secs).unwrap_or(20.0),
        manual,
        data: data.unwrap_or_else(|| PathBuf::from("data/valley.sqlite")),
        config,
        static_dir,
        seed,
        snapshot_every: snapshot_every.or(file.snapshot_every).unwrap_or(5),
    })
}

#[derive(Default)]
struct FileDefaults {
    port: Option<u16>,
    window_secs: Option<f64>,
    snapshot_every: Option<u32>,
}

fn read_server_defaults(path: &std::path::Path) -> FileDefaults {
    let Ok(text) = std::fs::read_to_string(path) else {
        return FileDefaults::default();
    };
    #[derive(serde::Deserialize)]
    struct File {
        server: Option<Server>,
    }
    #[derive(serde::Deserialize)]
    struct Server {
        port: Option<u16>,
        window_secs: Option<f64>,
        snapshot_every: Option<u32>,
    }
    let Ok(file) = toml::from_str::<File>(&text) else {
        return FileDefaults::default();
    };
    let Some(server) = file.server else {
        return FileDefaults::default();
    };
    FileDefaults {
        port: server.port,
        window_secs: server.window_secs,
        snapshot_every: server.snapshot_every,
    }
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn parse_next<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    let value = next_string(args, flag)?;
    value
        .parse()
        .map_err(|_| format!("{flag} could not parse {value}"))
}

const HELP: &str = "\
villmarks-server

  --port 18431
  --window-secs 20
  --manual
  --data data/valley.sqlite
  --config config/evening.toml
  --static web/dist
  --seed 1
  --snapshot-every 5
";
