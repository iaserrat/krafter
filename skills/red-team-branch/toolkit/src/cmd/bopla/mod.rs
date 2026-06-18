mod args;
mod field;
mod probe;
mod report;
mod request;

pub use args::Args;

use crate::config::Ctx;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    if args.fields.is_empty() {
        anyhow::bail!("no candidate fields — pass --field key=value");
    }
    let base_obj = field::base_object(&args.body)?;
    let client = ctx.client_for(args.as_profile.as_deref())?;
    let target = request::Target::build(&args, ctx)?;
    eprintln!(
        "[rtk] bopla: {} candidate fields -> {} (mutates local test target)",
        args.fields.len(),
        target.write_url
    );
    let mut results = Vec::new();
    let mut accepted = Vec::new();
    for raw in &args.fields {
        let (key, value) = field::parse(raw)?;
        let result = probe::run(&args, &client, &target, &base_obj, &key, &value).await;
        let accepted_field = result.status == probe::Status::Accepted;
        results.push(report::result(&key, &value, &result));
        if accepted_field {
            accepted.push(key);
        }
    }
    report::emit(args.as_profile, target, accepted, results);
    Ok(())
}

#[cfg(test)]
mod tests;
