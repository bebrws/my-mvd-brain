use clap::{ArgAction, Parser, Subcommand};

use crate::cmd_ask::AskArgs;
use crate::cmd_audit::AuditArgs;
use crate::cmd_binding::BindingArgs;
use crate::cmd_chat::ChatArgs;
use crate::cmd_config::ConfigArgs;
use crate::cmd_correct::CorrectArgs;
use crate::cmd_create::CreateArgs;
use crate::cmd_debug_segment::DebugSegmentArgs;
use crate::cmd_delete::DeleteArgs;
use crate::cmd_doctor::DoctorArgs;
use crate::cmd_enrich::EnrichArgs;
use crate::cmd_export::ExportArgs;
use crate::cmd_facts::FactsArgs;
use crate::cmd_find::FindArgs;
use crate::cmd_follow::FollowArgs;
use crate::cmd_lock::LockArgs;
use crate::cmd_memories::MemoriesArgs;
use crate::cmd_models::ModelsArgs;
use crate::cmd_nudge::NudgeArgs;
use crate::cmd_open::OpenArgs;
use crate::cmd_plan::PlanArgs;
use crate::cmd_process_queue::ProcessQueueArgs;
use crate::cmd_put::PutArgs;
use crate::cmd_put_many::PutManyArgs;
use crate::cmd_resolve::ResolveArgs;
use crate::cmd_schema::SchemaArgs;
use crate::cmd_session::SessionArgs;
use crate::cmd_sketch::SketchArgs;
use crate::cmd_state::StateArgs;
use crate::cmd_stats::StatsArgs;
use crate::cmd_status::StatusArgs;
use crate::cmd_tables::TablesArgs;
use crate::cmd_tickets::TicketsArgs;
use crate::cmd_timeline::TimelineArgs;
use crate::cmd_unlock::UnlockArgs;
use crate::cmd_update::UpdateArgs;
use crate::cmd_vec::VecArgs;
use crate::cmd_verify::VerifyArgs;
use crate::cmd_verify_single::VerifySingleFileArgs;
use crate::cmd_version::VersionArgs;
use crate::cmd_view::ViewArgs;
use crate::cmd_when::WhenArgs;
use crate::cmd_who::WhoArgs;
use crate::cmd_setup::SetupArgs;
use crate::cmd_usage::UsageArgs;

#[derive(Parser)]
#[command(
    name = "mvd",
    about = "Memvid single-file memory CLI",
    version = memvid_core::MEMVID_CORE_VERSION,
    propagate_version = true,
)]
pub struct Cli {
    /// Increase logging verbosity (use multiple times for more detail)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new `.mv2` memory file
    Create(CreateArgs),
    /// Interactive LLM chat REPL
    Chat(ChatArgs),
    /// Inspect metadata and manifests for an existing memory
    Open(OpenArgs),
    /// Append a frame to the memory, optionally with metadata
    Put(PutArgs),
    /// Store a correction with retrieval priority boost
    Correct(CorrectArgs),
    /// Batch ingest multiple documents with pre-computed embeddings
    PutMany(PutManyArgs),
    /// View a single frame
    View(ViewArgs),
    /// Update an existing frame
    Update(UpdateArgs),
    /// Delete a frame from the memory
    Delete(DeleteArgs),
    /// View the timeline of frames
    Timeline(TimelineArgs),
    /// Ask questions with retrieval + synthesis
    Ask(AskArgs),
    /// Generate an audit report with full source provenance
    Audit(AuditArgs),
    /// Perform lexical (BM25) exact-match search over the memory
    Find(FindArgs),
    /// Perform semantic (Cosine) vector similarity search over the memory
    Vec(VecArgs),
    /// Dump raw vector segment bytes for debugging
    DebugSegment(DebugSegmentArgs),
    /// Resolve temporal phrases and list matching frames
    When(WhenArgs),
    /// Display statistics about the memory
    Stats(StatsArgs),
    /// Run integrity verification checks
    Verify(VerifyArgs),
    /// Run doctor workflows to repair or optimise the memory
    Doctor(DoctorArgs),
    /// Process the enrichment queue (re-extract skim frames, update indexes)
    ProcessQueue(ProcessQueueArgs),
    /// Ensure no auxiliary files exist alongside the memory
    VerifySingleFile(VerifySingleFileArgs),
    /// Extract, list, export, and view tables from documents
    Tables(TablesArgs),
    /// Manage access tickets
    Tickets(TicketsArgs),
    /// View and manage your plan/subscription
    Plan(PlanArgs),
    /// Show memory binding information
    Binding(BindingArgs),
    /// Manage persistent CLI configuration (API keys, settings)
    Config(ConfigArgs),
    /// Show configuration and system status
    Status(StatusArgs),
    /// Show the active writer holding the lock
    Who(WhoArgs),
    /// Request the active writer flush and release when safe
    Nudge(NudgeArgs),
    /// Run enrichment engines to extract memory cards from frames
    Enrich(EnrichArgs),
    /// View extracted memory cards
    Memories(MemoriesArgs),
    /// Query current entity state (O(1) lookup)
    State(StateArgs),
    /// Audit fact changes with provenance and filtering
    Facts(FactsArgs),
    /// Export facts to N-Triples, JSON, or CSV format
    Export(ExportArgs),
    /// Infer and manage predicate schemas
    Schema(SchemaArgs),
    /// Manage LLM models for enrichment
    Models(ModelsArgs),
    /// Traverse the Logic-Mesh entity graph
    Follow(FollowArgs),
    /// Build and manage sketch track for fast candidate generation
    Sketch(SketchArgs),
    /// Manage time-travel replay sessions
    Session(SessionArgs),
    /// Encrypt a memory file into an encrypted capsule (.mv2e)
    Lock(LockArgs),
    /// Decrypt an encrypted capsule (.mv2e) back to a `.mv2` file
    Unlock(UnlockArgs),
    /// Print version information for debugging scripts
    Version(VersionArgs),
    /// Download all models and create the memory file for fully offline usage
    Setup(SetupArgs),
    /// Show how mvd has been used (which harness, which commands, which repos)
    Usage(UsageArgs),
    /// Resolve (and optionally create) the canonical memory-file path
    Resolve(ResolveArgs),
}
