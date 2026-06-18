use super::{event, log::HitLog, Args};
use std::io::Write;
use tiny_http::{Response, Server};

const WILDCARD_HOST: &str = "0.0.0.0";

pub fn serve(args: Args) -> anyhow::Result<()> {
    let addr = format!("{}:{}", args.host, args.port);
    let server = Server::http(&addr).map_err(|e| anyhow::anyhow!("bind {addr}: {e}"))?;
    announce(&args, &addr);
    let mut log = HitLog::open(args.log_file.as_ref())?;
    for mut request in server.incoming_requests() {
        let event = event::capture(&mut request, args.max_body);
        let line = event.to_string();
        println!("{line}");
        let _ = std::io::stdout().flush();
        log.write(&line);
        let status = tiny_http::StatusCode(args.respond_status);
        let response = Response::from_string(args.respond_body.clone()).with_status_code(status);
        let _ = request.respond(response);
    }
    Ok(())
}

fn announce(args: &Args, addr: &str) {
    eprintln!("[rtk] callback server listening on http://{addr}/");
    eprintln!("[rtk] point the target at e.g. http://{addr}/ssrf-canary-001 and watch stdout");
    if args.host == WILDCARD_HOST {
        eprintln!("[rtk][WARN] bound to 0.0.0.0 — reachable from other hosts on the network");
    }
    if let Some(file) = &args.log_file {
        eprintln!("[rtk] logging hits to {}", file.display());
    }
}
