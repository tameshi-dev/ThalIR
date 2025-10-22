use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "thalir")]
#[command(about = "ThalIR - Privacy-preserving IR for smart contract security analysis")]
#[command(version = "0.1.0")]
#[command(author = "Gianluca Brigandi <gbrigand@gmail.com>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        annotated: bool,

        #[arg(long, requires = "annotated")]
        ascii: bool,

        #[arg(long, value_enum, default_value = "none")]
        obfuscate: ObfuscationLevel,

        #[arg(long, requires = "obfuscate")]
        save_mapping: Option<PathBuf>,

        #[arg(short, long)]
        verbose: bool,
    },

    Deobfuscate {
        #[arg(short, long)]
        mapping: PathBuf,

        #[arg(short, long)]
        report: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Validate {
        input: PathBuf,

        #[arg(short, long)]
        verbose: bool,
    },

    Debug {
        input: PathBuf,

        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ObfuscationLevel {
    None,
    Minimal,
    Standard,
}

impl From<ObfuscationLevel> for thalir_core::ObfuscationLevel {
    fn from(level: ObfuscationLevel) -> Self {
        match level {
            ObfuscationLevel::None => thalir_core::ObfuscationLevel::None,
            ObfuscationLevel::Minimal => thalir_core::ObfuscationLevel::Minimal,
            ObfuscationLevel::Standard => thalir_core::ObfuscationLevel::Standard,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            annotated,
            ascii,
            obfuscate,
            save_mapping,
            verbose,
        } => cmd_compile(
            input,
            output,
            annotated,
            ascii,
            obfuscate,
            save_mapping,
            verbose,
        ),
        Commands::Deobfuscate {
            mapping,
            report,
            output,
        } => cmd_deobfuscate(mapping, report, output),
        Commands::Validate { input, verbose } => cmd_validate(input, verbose),
        Commands::Debug { input, verbose } => cmd_debug(input, verbose),
    }
}

fn cmd_compile(
    input: PathBuf,
    output: Option<PathBuf>,
    annotated: bool,
    ascii: bool,
    obfuscate: ObfuscationLevel,
    save_mapping: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    use colored::*;
    use std::fs;
    use std::time::Instant;
    use thalir_core::ObfuscationConfig;
    use thalir_emit::{AnnotatedIREmitter, ThalIREmitter};
    use thalir_transform::transform_solidity_to_ir_with_filename;

    if verbose {
        println!("{}", " ThalIR Compiler".bright_blue().bold());
        println!("{}", "=".repeat(50).bright_blue());
        println!(" Input: {}", input.display());
        if let Some(ref out) = output {
            println!(" Output: {}", out.display());
        }
        if annotated {
            println!(
                " Mode: Annotated ThalIR{}",
                if ascii { " (ASCII)" } else { "" }
            );
        }
        if !matches!(obfuscate, ObfuscationLevel::None) {
            println!(" Obfuscation: {:?}", obfuscate);
        }
        println!();
    }

    let start = Instant::now();

    if verbose {
        println!(" Loading Solidity source...");
    }
    let solidity_content = fs::read_to_string(&input)?;
    let filename = input.to_str();

    if verbose {
        println!(" Transforming to ThalIR...");
    }
    let contracts = transform_solidity_to_ir_with_filename(&solidity_content, filename)?;

    if contracts.is_empty() {
        println!("{}", "  No contracts found in input".yellow());
        return Ok(());
    }

    if verbose {
        println!(" Generating IR output...");
    }

    let ir_output = match (annotated, matches!(obfuscate, ObfuscationLevel::None)) {
        (true, true) => {
            use thalir_emit::annotated_ir_emitter::AnnotationConfig;
            let config = AnnotationConfig {
                emit_position_markers: true,
                emit_visual_cues: true,
                use_ascii_cues: ascii,
                emit_ordering_analysis: true,
                emit_function_headers: true,
            };
            let emitter = AnnotatedIREmitter::new(contracts).with_annotation_config(config);
            (emitter.emit_to_string(false), None)
        }
        (true, false) => {
            use thalir_emit::annotated_ir_emitter::AnnotationConfig;
            let obf_config = ObfuscationConfig {
                level: obfuscate.into(),
                retain_mapping: save_mapping.is_some(),
                hash_salt: None,
                strip_string_constants: true,
                strip_error_messages: true,
                strip_metadata: true,
            };
            let ann_config = AnnotationConfig {
                emit_position_markers: true,
                emit_visual_cues: true,
                use_ascii_cues: ascii,
                emit_ordering_analysis: true,
                emit_function_headers: true,
            };
            let (emitter, mapping) =
                AnnotatedIREmitter::with_obfuscation(contracts, obf_config, ann_config)?;
            (emitter.emit_to_string(false), mapping)
        }
        (false, true) => {
            let emitter = ThalIREmitter::new(contracts);
            (emitter.emit_to_string(false), None)
        }
        (false, false) => {
            let obf_config = ObfuscationConfig {
                level: obfuscate.into(),
                retain_mapping: save_mapping.is_some(),
                hash_salt: None,
                strip_string_constants: true,
                strip_error_messages: true,
                strip_metadata: true,
            };
            let (emitter, mapping) = ThalIREmitter::with_obfuscation(contracts, obf_config)?;
            (emitter.emit_to_string(false), mapping)
        }
    };

    if let (Some(mapping_path), Some(mapping)) = (save_mapping, ir_output.1) {
        if verbose {
            println!(" Saving obfuscation mapping...");
        }
        let mapping_json = serde_json::to_string_pretty(&mapping)?;
        fs::write(&mapping_path, mapping_json)?;
        if verbose {
            println!("   Saved to: {}", mapping_path.display());
        }
    }

    if let Some(output_path) = output {
        fs::write(&output_path, &ir_output.0)?;
        if verbose {
            let elapsed = start.elapsed();
            println!(
                "\n {} Compilation successful!",
                "SUCCESS:".bright_green().bold()
            );
            println!("   Time: {:.3}s", elapsed.as_secs_f64());
            println!("   Output: {}", output_path.display());
        }
    } else {
        println!("{}", ir_output.0);
    }

    Ok(())
}

fn cmd_deobfuscate(
    mapping: PathBuf,
    report: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    use colored::*;
    use std::fs;
    use thalir_core::{ObfuscationMapping, VulnerabilityMapper};

    let mapping_json = fs::read_to_string(&mapping)?;
    let obf_mapping: ObfuscationMapping = serde_json::from_str(&mapping_json)?;
    let mapper = VulnerabilityMapper::from_mapping(obf_mapping);

    let report_content = if let Some(report_path) = report {
        fs::read_to_string(&report_path)?
    } else {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let deobfuscated = mapper.deobfuscate_report(&report_content);

    if let Some(output_path) = output {
        fs::write(&output_path, &deobfuscated)?;
        println!(
            " {} De-obfuscated report saved to: {}",
            "SUCCESS:".bright_green().bold(),
            output_path.display()
        );
    } else {
        println!("{}", deobfuscated);
    }

    Ok(())
}

fn cmd_validate(input: PathBuf, verbose: bool) -> Result<()> {
    use colored::*;
    use std::fs;

    if verbose {
        println!("{}", " Validating ThalIR".bright_cyan().bold());
        println!("{}", "".repeat(50).bright_cyan());
        println!(" Input: {}", input.display());
        println!();
    }

    let ir_content = fs::read_to_string(&input)?;

    if verbose {
        println!(" Parsing with Pest parser...");
    }

    match thalir_parser::parse(&ir_content) {
        Ok(pairs) => {
            let count = pairs.count();
            println!("{}", " VALID".bright_green().bold());
            if verbose {
                println!("   Parsed {} top-level elements", count);
            }
            Ok(())
        }
        Err(e) => {
            println!("{}", " INVALID".bright_red().bold());
            println!("\n{}", "Parse Error:".bright_red());
            println!("{}", e);
            Err(anyhow::anyhow!("Validation failed"))
        }
    }
}

fn cmd_debug(input: PathBuf, verbose: bool) -> Result<()> {
    use colored::*;
    use std::fs;
    use thalir_transform::transform_solidity_to_ir;

    if verbose {
        println!("{}", " Debug IR Dump".bright_cyan().bold());
        println!("{}", "=".repeat(50).bright_cyan());
        println!(" Input: {}", input.display());
        println!();
    }

    let solidity_content = fs::read_to_string(&input)?;
    let contracts = transform_solidity_to_ir(&solidity_content)?;

    if contracts.is_empty() {
        println!("  No contracts found");
        return Ok(());
    }

    println!(" Found {} contract(s)\n", contracts.len());

    for (idx, contract) in contracts.iter().enumerate() {
        println!(
            "{}",
            format!(" Contract {}: {}", idx, contract.name)
                .bright_green()
                .bold()
        );
        println!("{}", "".repeat(60).bright_green());
        println!("Functions: {}", contract.functions.len());
        println!("Storage slots: {}", contract.storage_layout.slots.len());

        if verbose {
            if !contract.storage_layout.slots.is_empty() {
                println!("\n  Storage Layout:");
                for slot in &contract.storage_layout.slots {
                    println!(
                        "    slot {} = {}: {:?}",
                        slot.slot, slot.name, slot.var_type
                    );
                }
            }

            for (func_name, function) in &contract.functions {
                println!(
                    "\n  {}",
                    format!(" Function: {}", func_name).bright_yellow()
                );
                println!("     Visibility: {:?}", function.visibility);
                println!("     Mutability: {:?}", function.mutability);
                println!("     Parameters: {}", function.signature.params.len());
                println!("     Returns: {}", function.signature.returns.len());
                println!("     Blocks: {}", function.body.blocks.len());

                for (block_id, block) in &function.body.blocks {
                    println!(
                        "       Block {:?}: {} instructions",
                        block_id,
                        block.instructions.len()
                    );
                }
            }
        }

        println!();
    }

    Ok(())
}
