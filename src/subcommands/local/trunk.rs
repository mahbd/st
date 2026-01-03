//! `trunk` subcommand.

use crate::{ctx::StContext, errors::StResult};
use clap::{Args, Subcommand};
use nu_ansi_term::Color;

/// CLI arguments for the `trunk` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct TrunkCmd {
    #[clap(subcommand)]
    pub command: TrunkSubcommand,
}

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum TrunkSubcommand {
    /// List all trunk branches
    #[clap(visible_alias = "ls")]
    List,
    /// Switch to a different trunk
    #[clap(visible_alias = "sw")]
    Switch {
        /// Name of the trunk to switch to
        trunk_name: String,
    },
    /// Add a new trunk branch
    Add {
        /// Name of the new trunk branch
        trunk_name: String,
    },
    /// Remove a trunk branch
    #[clap(visible_alias = "rm")]
    Remove {
        /// Name of the trunk to remove
        trunk_name: String,
    },
}

impl TrunkCmd {
    /// Run the `trunk` subcommand.
    pub fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        match &self.command {
            TrunkSubcommand::List => self.list(&ctx),
            TrunkSubcommand::Switch { trunk_name } => self.switch(&mut ctx, trunk_name),
            TrunkSubcommand::Add { trunk_name } => self.add(&mut ctx, trunk_name),
            TrunkSubcommand::Remove { trunk_name } => self.remove(&mut ctx, trunk_name),
        }
    }

    fn list(&self, ctx: &StContext<'_>) -> StResult<()> {
        let trunks = ctx.tree.list_trunks();
        let active = ctx.tree.trunk_name();

        if trunks.is_empty() {
            println!("No trunks configured.");
            return Ok(());
        }

        println!("Trunk branches:");
        for trunk in trunks {
            if trunk == active {
                println!("  {} {}", Color::Green.paint("*"), Color::Green.bold().paint(&trunk));
            } else {
                println!("    {}", trunk);
            }
        }
        Ok(())
    }

    fn switch(&self, ctx: &mut StContext<'_>, trunk_name: &str) -> StResult<()> {
        ctx.tree.switch_trunk(trunk_name)?;
        println!(
            "Switched to trunk `{}`",
            Color::Green.paint(trunk_name)
        );
        Ok(())
    }

    fn add(&self, ctx: &mut StContext<'_>, trunk_name: &str) -> StResult<()> {
        // Check if the branch exists in the repository
        if ctx.repository.find_branch(trunk_name, git2::BranchType::Local).is_err() {
            println!(
                "{}: Branch `{}` does not exist in the repository.",
                Color::Red.paint("Error"),
                trunk_name
            );
            return Ok(());
        }

        ctx.tree.add_trunk(trunk_name.to_string());
        println!(
            "Added trunk `{}`. Use `{}` to switch to it.",
            Color::Green.paint(trunk_name),
            Color::Blue.paint(format!("st trunk switch {}", trunk_name))
        );
        Ok(())
    }

    fn remove(&self, ctx: &mut StContext<'_>, trunk_name: &str) -> StResult<()> {
        // Confirm removal
        let confirm = inquire::Confirm::new(
            format!(
                "Remove trunk `{}` and all its tracked branches?",
                Color::Yellow.paint(trunk_name)
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }

        ctx.tree.remove_trunk(trunk_name)?;
        println!(
            "Removed trunk `{}`",
            Color::Red.paint(trunk_name)
        );
        Ok(())
    }
}
