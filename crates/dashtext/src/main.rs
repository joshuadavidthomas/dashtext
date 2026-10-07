//! Dashtext: quick capture for plain text, inspired by Drafts.

// gpui-fast's macros emit `gpui::` paths, where gpui-pre's emit `::gpui_kit::`.
extern crate gpui_kit as gpui;

mod app;
mod assets;
mod capture;
mod cli;
mod commands;
mod drafts;
mod instance;
mod library;
mod menus;
mod paths;
#[cfg(test)]
mod test_support;
mod time_format;

use std::process::ExitCode;

use anyhow::Context as _;
use clap::Parser as _;
use dashtext_core::Store;
use gpui_kit::component::Theme;

use crate::assets::AppAssets;
use crate::cli::Cli;
use crate::cli::Invocation;
use crate::instance::Claim;
use crate::instance::Request;
use crate::library::Library;
use crate::paths::Paths;

fn main() -> ExitCode {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,dashtext=info"),
    )
    .init();

    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dashtext: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let paths = Paths::resolve()?;
    paths.ensure()?;

    match cli.run(&paths)? {
        Invocation::Done => Ok(()),
        Invocation::Gui(request) => match instance::claim(&paths, request)? {
            Claim::Forwarded => Ok(()),
            Claim::Primary(listener) => run_gui(&paths, listener, request),
        },
    }
}

fn run_gui(paths: &Paths, listener: instance::Listener, request: Request) -> anyhow::Result<()> {
    // Answer other launches right away; requests queue until the app runs.
    // Serving only after opening the library would let a slow open look like
    // a stale socket to a second launch.
    let (sender, requests) = async_channel::unbounded();
    let socket = listener.serve(sender)?;

    let library_path = paths.library();
    let store = Store::open(&library_path).with_context(|| {
        format!(
            "could not open the draft library at {}",
            library_path.display()
        )
    })?;

    gpui_kit::application()
        .with_assets(AppAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            cx.set_app_identity(app::APP_ID, "Dashtext");
            Theme::sync_system_appearance(None, cx);
            commands::bind_keys(cx);
            Library::init(store, cx);
            app::init(cx);

            // Remove the socket on the way out so the next launch starts cleanly.
            let mut socket = Some(socket);
            cx.on_app_quit(move |_| {
                drop(socket.take());
                async {}
            })
            .detach();

            cx.spawn(async move |cx| {
                while let Ok(request) = requests.recv().await {
                    cx.update(|cx| app::handle(request, cx));
                }
            })
            .detach();

            app::handle(request, cx);
            cx.activate(true);
        });

    Ok(())
}
