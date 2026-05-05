use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct MemoriesArgs {
    pub file: PathBuf,
    /// Filter by entity name
    #[arg(long)]
    pub entity: Option<String>,
    /// Filter by slot (requires --entity)
    #[arg(long)]
    pub slot: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: MemoriesArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let mem_stats = mem.memories_stats();

    if let Some(ref entity) = args.entity {
        if let Some(ref slot) = args.slot {
            // Show specific entity:slot
            let current = mem.get_current_memory(entity, slot);
            let all_values = mem.aggregate_memory_slot(entity, slot);

            if args.json {
                let obj = serde_json::json!({
                    "entity": entity,
                    "slot": slot,
                    "current": current.map(|c| serde_json::json!({
                        "value": c.value,
                        "confidence": c.confidence,
                        "kind": c.kind.as_str(),
                    })),
                    "all_values": all_values,
                    "count": mem.count_memory_occurrences(entity, slot, None),
                });
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                println!("Entity: {entity}");
                println!("Slot:   {slot}");
                if let Some(card) = current {
                    println!("Value:  {}", card.value);
                    println!("Kind:   {}", card.kind.as_str());
                    println!("Confidence: {:.2}", card.confidence.unwrap_or(0.0));
                } else {
                    println!("Value:  (none)");
                }
                if all_values.len() > 1 {
                    println!("\nAll values ({}):", all_values.len());
                    for v in &all_values {
                        println!("  - {v}");
                    }
                }
            }
        } else {
            // Show all slots for an entity
            let cards = mem.get_entity_memories(entity);
            if args.json {
                let items: Vec<_> = cards
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "slot": c.slot,
                            "value": c.value,
                            "kind": c.kind.as_str(),
                            "confidence": c.confidence,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                println!("Entity: {entity} ({} cards)", cards.len());
                let mut last_slot = String::new();
                for card in &cards {
                    if card.slot != last_slot {
                        println!("\n  {}:", card.slot);
                        last_slot = card.slot.clone();
                    }
                    println!("    {} ({})", card.value, card.kind.as_str());
                }
            }
        }
    } else {
        // Show overall stats
        let entities = mem.memory_entities();
        if args.json {
            println!("{}", serde_json::to_string_pretty(&mem_stats)?);
        } else {
            println!("Memories for {}", args.file.display());
            println!("  Cards:           {}", mem_stats.card_count);
            println!("  Entities:        {}", mem_stats.entity_count);
            println!("  Slots:           {}", mem_stats.slot_count);
            println!("  Enriched frames: {}", mem_stats.enriched_frames);
            if !mem_stats.cards_by_kind.is_empty() {
                println!("  By kind:");
                for (kind, count) in &mem_stats.cards_by_kind {
                    println!("    {kind}: {count}");
                }
            }
            if !entities.is_empty() {
                println!("  Entities:");
                for e in &entities {
                    let cards = mem.get_entity_memories(e);
                    println!("    {e} ({} cards)", cards.len());
                }
            }
        }
    }
    Ok(())
}
