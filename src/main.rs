#![feature(let_chains)]

use std::env;

// use chrono::offset::TimeZone;
use chrono::FixedOffset;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::command::{Command, CommandOptionType, CommandType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{Error, Result};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy)]
enum TimestampFormat {
    Relative,
    ShortTime,
    LongTime,
    ShortDate,
    LongDate,
    Full,
    FullWithDOW,
}
impl TimestampFormat {
    fn from_marker(marker: &str) -> Self {
        match marker {
            "t" => Self::ShortTime,
            "T" => Self::LongTime,
            "d" => Self::ShortDate,
            "D" => Self::LongDate,
            "f" => Self::Full,
            "F" => Self::FullWithDOW,
            "R" => Self::Relative,
            _ => panic!("only \"tTdDfFR\" are allowed as markers"),
        }
    }
    fn marker(&self) -> &str {
        match self {
            TimestampFormat::Relative => "R",
            TimestampFormat::ShortTime => "t",
            TimestampFormat::LongTime => "T",
            TimestampFormat::ShortDate => "d",
            TimestampFormat::LongDate => "D",
            TimestampFormat::Full => "f",
            TimestampFormat::FullWithDOW => "F",
        }
    }
}

/// Parses the given timezone descriptor, using the provided
/// default timezone for the calculation.
fn parse_timezone(_tz: &str, default: FixedOffset) -> FixedOffset {
    default
}

/// Renders a Discord datetime badge.
fn format_badge(timestamp: i64, format: TimestampFormat) -> String {
    format!("<t:{}:{}>", timestamp, format.marker())
}

const RCLICK_NAME: &str = "Tell me the times";
const DEV_MENTION: &str = "@ilonachan#2286";
const DEV_PRONOUNS: (&str, &str, &str) = ("she", "her", "her");

async fn run_rclick(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    send_command_response(ctx, interaction, "hi".to_string(), true).await
}
async fn run_timestamp(ctx: &Context, interaction: &ApplicationCommandInteraction) -> Result<()> {
    let mut descriptor = None;
    let mut timezone = Some("default".to_string());
    let mut format = Some("\"R\"".to_string());
    let mut list = Some(true);

    for opt in &interaction.data.options {
        let v = opt.value.as_ref();
        match opt.name.as_str() {
            "descriptor" => descriptor = v.map(|v| v.to_string()).or(descriptor),
            "timezone" => timezone = v.map(|v| v.to_string()).or(timezone),
            "format" => format = v.map(|v| v.to_string()).or(format),
            "list" => list = v.and_then(|v| v.as_bool()).or(list),
            _ => (),
        }
    }

    let time_naive = chrono::Local::now().naive_local();

    let default_timezone =
        FixedOffset::east_opt(3600).expect("the timezone in the database should be correct");

    let timezone = timezone.expect("either the default or a user value should exist here");
    let timezone = parse_timezone(
        timezone.get(1..timezone.len() - 1).unwrap_or("default"),
        default_timezone,
    );

    let time = time_naive.and_local_timezone(timezone).single().unwrap();

    let format = format.expect("either the default or a user value should exist here");
    if format.len() != 3 {
        error!("Format choice returned invalid value: {}", format);
        return Err(Error::Other("Format choice returned invalid value"));
    }
    let format = format.chars().nth(1).unwrap();
    if !"RtTdDfF".contains(format) {
        error!("Format choice returned invalid value: {}", format);
        return Err(Error::Other("Format choice returned invalid value"));
    }
    let format = TimestampFormat::from_marker(format.to_string().as_str());

    let badge = format_badge(time.timestamp(), format);

    let mut final_message = format!("`{0}` => {0}", badge);
    let list = list.expect("either the default or a user value should exist here");
    if list {
        final_message += "\n__Other options:__\n**R**elative, short **t**ime, long **T**ime, short **d**ate, long **D**ate, **f**ull datetime, **F**ull datetime with Day-of-Week"
    }

    send_command_response(ctx, interaction, final_message, false).await
}

fn register_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
        .create_application_command(|command| command.name(RCLICK_NAME).kind(CommandType::Message))
        .create_application_command(|command| {
            command
                .name("timestamp")
                .description(
                    "Converts the given description of a time/date into a discord timestamp badge",
                )
                .dm_permission(true)
                .kind(CommandType::ChatInput)
                .create_option(|o| {
                    o.name("descriptor").kind(CommandOptionType::String).required(true).set_autocomplete(false)
                    .description("The datetime descriptor (e.g. \"Apr 16\", \"6pm\", \"23:45\", \"twenty minutes ago\")")
                })
                .create_option(|o| {
                    o.name("timezone").kind(CommandOptionType::String).required(false).set_autocomplete(false) //TODO: Autocomplete makes sense here
                    // (default: "default", use your timezone stored in the database. Ex: "utc+3", "-4", "default+11", "pt", "EST")
                    .description("The timezone relative to which the datetime descriptors should be interpreted")
                })
                .create_option(|o| {
                    o.name("format").kind(CommandOptionType::String)
                    .description("The format of string that should be returned")
                    .add_string_choice("relative", "R")
                    .add_string_choice("short time", "t").add_string_choice("long time", "T")
                    .add_string_choice("short date", "d").add_string_choice("long date", "D")
                    .add_string_choice("long date with short time", "f")
                    .add_string_choice("long date with day of week and short time", "F")
                })
                .create_option(|o| o.name("list").kind(CommandOptionType::Boolean)
                .description("Along with the ready-made timestamp, list the other format options (default: true)"))
        })
}

async fn send_command_response(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
    content: String,
    public: bool
) -> Result<()> {
    interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content).ephemeral(!public))
        })
        .await
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        if let Ok(id_str) = env::var("GUILD_ID") {
            let guild_id = GuildId(id_str.parse().expect("guild id must be an integer"));

            match GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                register_commands(commands)
            })
            .await
            {
                Ok(commands) => debug!(
                    "registered application commands for guild {}: {:?}",
                    guild_id, commands
                ),
                Err(why) => error!(
                    "application commands for guild {} coult not be registered: {:?}",
                    guild_id, why
                ),
            }
        } else {
            match Command::set_global_application_commands(&ctx.http, |commands| {
                register_commands(commands)
            })
            .await
            {
                Ok(commands) => debug!("registered application commands globally: {:?}", commands),
                Err(why) => error!(
                    "global application commands could not be registered: {:?}",
                    why
                ),
            }
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!("Received command interaction: {:#?}", command);

            if let Err(why) = match command.data.name.as_str() {
                RCLICK_NAME => run_rclick(&ctx, &command).await,
                "timestamp" => run_timestamp(&ctx, &command).await,
                _ => {
                    error!("unexpected interaction name from discord: {}", command.data.name);
                    Err(Error::Other("unexpected interaction name from discord"))
                },
            } {
                error!("cannot respond to slash command: {}", why);
                if let Err(why) = send_command_response(
                    &ctx,
                    &command,
                    format!(
                        "An error happened on the server side. \
                Please contact {} and tell {} what command you ran at what time.",
                        DEV_MENTION, DEV_PRONOUNS.2
                    ), false
                )
                .await
                {
                    error!("cannot send error message to user: {}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::from_filename(
        if let Ok(Ok(true)) = env::var("TT_DEBUG").map(|val| val.parse()) {
            "dev.env"
        } else {
            "deploy.env"
        },
    )
    .expect("Failed to load .env file");

    tracing_subscriber::fmt::init();

    // invite link:
    // https://discord.com/api/oauth2/authorize?client_id=1086383822070890536&permissions=277562272832&scope=applications.commands%20bot

    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN environment variable");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
}
