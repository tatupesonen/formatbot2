use std::time::Duration;

use format_core::*;
use futures::future::join_all;
use poise::serenity_prelude::{
    self, ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, Message,
};
use regex::Regex;
use tokio::time::{self, timeout};

use crate::{Context, Error};

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "All available FormatBot options",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Format some code
#[poise::command(context_menu_command = "format")]
pub async fn format(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await?;

    let re = Regex::new(r"(?s)```(\w*)\n(.*?)\n```").unwrap();

    let captures: Vec<format_core::CodeBlock> = re
        .captures_iter(&message.content)
        .map(|caps| {
            let lang = caps.get(1).and_then(|m| {
                let s = m.as_str();
                if s.is_empty() { None } else { Some(s) }
            });

            let code = caps.get(2).map(|m| m.as_str());

            format_core::CodeBlock {
                code,
                language: lang,
            }
        })
        .collect();

    // Get available formatters
    let guard = ctx.data().enabled_formatters.read().await;
    let mapped: Vec<(&str, &str, &str)> = captures
        .iter()
        .filter_map(|block| {
            if let (Some(lang), Some(code)) = (block.language, block.code) {
                guard.get(lang).map(|server| (lang, code, server.as_str()))
            } else {
                None
            }
        })
        .collect();

    let client = reqwest::Client::new();
    let timeout_duration = Duration::from_secs(3);

    // Now include lang with each request
    let all_requests = mapped.iter().map(|(lang, code, server)| {
        let request = client
            .post(format!("{}/format", server))
            .body(code.to_string())
            .send();

        // Return the language + the result future
        async move {
            match timeout(timeout_duration, request).await {
                Ok(Ok(resp)) => match resp.text().await {
                    Ok(body) => Some((lang.to_string(), body)),
                    Err(_) => None,
                },
                _ => None,
            }
        }
    });

    let results: Vec<Option<(String, String)>> = join_all(all_requests).await;

    // Construct final response
    let mut resp = String::new();
    for result in results.into_iter().flatten() {
        let (lang, body) = result;
        resp.push_str(&format!("```{}\n{}\n```\n\n", lang, body));
    }

    // Send final message
    ctx.say(resp).await?;
    Ok(())
}
