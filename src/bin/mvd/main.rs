mod cli;
mod common;
mod llm;

mod cmd_create;
mod cmd_open;
mod cmd_put;
mod cmd_correct;
mod cmd_put_many;
mod cmd_api_fetch;
mod cmd_view;
mod cmd_update;
mod cmd_delete;
mod cmd_timeline;
mod cmd_ask;
mod cmd_audit;
mod cmd_find;
mod cmd_vec_search;
mod cmd_debug_segment;
mod cmd_when;
mod cmd_stats;
mod cmd_verify;
mod cmd_doctor;
mod cmd_process_queue;
mod cmd_verify_single;
mod cmd_tables;
mod cmd_tickets;
mod cmd_plan;
mod cmd_binding;
mod cmd_config;
mod cmd_status;
mod cmd_who;
mod cmd_nudge;
mod cmd_enrich;
mod cmd_memories;
mod cmd_state;
mod cmd_facts;
mod cmd_export;
mod cmd_schema;
mod cmd_models;
mod cmd_follow;
mod cmd_sketch;
mod cmd_session;
mod cmd_lock;
mod cmd_unlock;
mod cmd_version;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = match cli.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .init();

    let result = dispatch(cli);

    if let Err(err) = result {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn dispatch(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Create(args) => cmd_create::run(args),
        Command::Open(args) => cmd_open::run(args),
        Command::Put(args) => cmd_put::run(args),
        Command::Correct(args) => cmd_correct::run(args),
        Command::PutMany(args) => cmd_put_many::run(args),
        Command::ApiFetch(args) => cmd_api_fetch::run(args),
        Command::View(args) => cmd_view::run(args),
        Command::Update(args) => cmd_update::run(args),
        Command::Delete(args) => cmd_delete::run(args),
        Command::Timeline(args) => cmd_timeline::run(args),
        Command::Ask(args) => cmd_ask::run(args),
        Command::Audit(args) => cmd_audit::run(args),
        Command::Find(args) => cmd_find::run(args),
        Command::VecSearch(args) => cmd_vec_search::run(args),
        Command::DebugSegment(args) => cmd_debug_segment::run(args),
        Command::When(args) => cmd_when::run(args),
        Command::Stats(args) => cmd_stats::run(args),
        Command::Verify(args) => cmd_verify::run(args),
        Command::Doctor(args) => cmd_doctor::run(args),
        Command::ProcessQueue(args) => cmd_process_queue::run(args),
        Command::VerifySingleFile(args) => cmd_verify_single::run(args),
        Command::Tables(args) => cmd_tables::run(args),
        Command::Tickets(args) => cmd_tickets::run(args),
        Command::Plan(args) => cmd_plan::run(args),
        Command::Binding(args) => cmd_binding::run(args),
        Command::Config(args) => cmd_config::run(args),
        Command::Status(args) => cmd_status::run(args),
        Command::Who(args) => cmd_who::run(args),
        Command::Nudge(args) => cmd_nudge::run(args),
        Command::Enrich(args) => cmd_enrich::run(args),
        Command::Memories(args) => cmd_memories::run(args),
        Command::State(args) => cmd_state::run(args),
        Command::Facts(args) => cmd_facts::run(args),
        Command::Export(args) => cmd_export::run(args),
        Command::Schema(args) => cmd_schema::run(args),
        Command::Models(args) => cmd_models::run(args),
        Command::Follow(args) => cmd_follow::run(args),
        Command::Sketch(args) => cmd_sketch::run(args),
        Command::Session(args) => cmd_session::run(args),
        Command::Lock(args) => cmd_lock::run(args),
        Command::Unlock(args) => cmd_unlock::run(args),
        Command::Version(args) => cmd_version::run(args),
    }
}
