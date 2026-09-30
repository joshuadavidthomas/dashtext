use std::io::IsTerminal as _;
use std::io::Read as _;

use anyhow::Context as _;
use anyhow::bail;
use clap::Parser;
use clap::Subcommand;
use dashtext_core::Folder;
use dashtext_core::Store;

use crate::instance;
use crate::instance::Request;
use crate::paths::Paths;

/// Quick capture for plain text.
///
/// Run without a command to open the drafts window. If Dashtext is already
/// running, the running instance handles the request.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Open the drafts window (the default).
    Open,
    /// Open the quick capture window.
    ///
    /// Bind this to a global shortcut in your desktop environment for
    /// system-wide capture.
    Capture,
    /// Add a draft to the inbox without opening a window.
    New {
        /// The draft's text. Read from standard input when omitted.
        text: Vec<String>,
        /// Flag the new draft.
        #[arg(long)]
        flag: bool,
    },
}

pub enum Invocation {
    /// Start the GUI (or hand the request to a running instance).
    Gui(Request),
    /// The command ran to completion without the GUI.
    Done,
}

impl Cli {
    pub fn run(self, paths: &Paths) -> anyhow::Result<Invocation> {
        match self.command.unwrap_or(Command::Open) {
            Command::Open => Ok(Invocation::Gui(Request::Open)),
            Command::Capture => Ok(Invocation::Gui(Request::Capture)),
            Command::New { text, flag } => {
                new_draft(paths, &text, flag)?;
                Ok(Invocation::Done)
            }
        }
    }
}

fn new_draft(paths: &Paths, text: &[String], flag: bool) -> anyhow::Result<()> {
    let content = if text.is_empty() {
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            bail!("provide the draft text as arguments or on standard input");
        }
        let mut content = String::new();
        stdin
            .read_to_string(&mut content)
            .context("could not read standard input")?;
        content.trim_end().to_owned()
    } else {
        text.join(" ")
    };

    if content.trim().is_empty() {
        bail!("the draft is empty");
    }

    let store = Store::open(&paths.library()).context("could not open the draft library")?;
    let draft = store.create_draft(&content, Folder::Inbox, flag)?;
    println!("{}", draft.id());

    // Let a running instance show the new draft; nothing to do otherwise.
    instance::notify(paths, Request::Reload);
    Ok(())
}
