use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;

use blueprint::{AdviceSelection, demo_renderer};
use clap::Parser;
use discovery::DiscoveredMiner;
use live::LiveMinerStrategy;
use myosu_games_liars_dice::{LiarsDiceRenderer, LiarsDiceSnapshot};
use myosu_games_poker::NlheRenderer;
use myosu_tui::events::UpdateEvent;
use myosu_tui::{GameRenderer, InteractionState, PipeMode, Screen, Shell};
use subtensor_runtime_common::NetUid;
use tokio::sync::mpsc;

const DEFAULT_TICK_RATE: Duration = Duration::from_millis(100);

mod blueprint;
mod cli;
mod discovery;
mod live;

use crate::cli::{AdviceArgs, Cli, DiscoveryRequest, GameSelection, Mode};

#[derive(Clone, Debug)]
struct DiscoverySelection {
    source: &'static str,
    detail: Option<String>,
    miner: Option<DiscoveredMiner>,
}

#[derive(Clone, Debug)]
struct LiveQuerySelection {
    source: &'static str,
    detail: Option<String>,
    result: Option<LiveMinerStrategy>,
}

struct RenderContext {
    advice: AdviceSelection,
    discovery: DiscoverySelection,
    live_query: LiveQuerySelection,
}

enum PipeResponse {
    Action(String),
    Clarify(String),
    Error(String),
    Quit(String),
}

struct LiveAdviceRefresh {
    stop: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiveAdviceConnectivity {
    Fresh,
    Stale,
    Offline,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let discovery_request = DiscoveryRequest::from_cli(&cli)?;

    if cli.smoke_test {
        return run_smoke_test(
            cli.game,
            AdviceArgs {
                checkpoint: cli.smoke_checkpoint,
                encoder_dir: cli.smoke_encoder_dir,
            },
            discovery_request,
            cli.require_artifact,
            cli.require_discovery,
            cli.require_live_query,
        )
        .await;
    }

    match cli.command {
        Some(Mode::Train(args)) => run_train(cli.game, args, discovery_request).await,
        Some(Mode::Pipe(args)) => run_pipe(cli.game, args, discovery_request).await,
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected subcommand (`train` or `pipe`) or --smoke-test",
        )),
    }
}

async fn run_smoke_test(
    game: GameSelection,
    args: AdviceArgs,
    discovery_request: DiscoveryRequest,
    require_artifact: bool,
    require_discovery: bool,
    require_live_query: bool,
) -> io::Result<()> {
    let context = resolve_context(game, args, discovery_request).await?;
    print!(
        "{}",
        smoke_report(
            &context,
            require_artifact,
            require_discovery,
            require_live_query,
        )?
    );
    Ok(())
}

fn smoke_report(
    context: &RenderContext,
    require_artifact: bool,
    require_discovery: bool,
    require_live_query: bool,
) -> io::Result<String> {
    if context.advice.game == GameSelection::LiarsDice {
        return liars_dice_smoke_report(
            context,
            require_artifact,
            require_discovery,
            require_live_query,
        );
    }

    ensure_artifact_selection(&context.advice, require_artifact)?;
    ensure_discovery_selection(&context.discovery, require_discovery)?;
    ensure_live_query_selection(&context.live_query, require_live_query)?;

    let initial = context.advice.surface.renderer().pipe_output();
    if !initial.contains("street=PREFLOP") {
        return Err(io::Error::other(format!(
            "smoke expected preflop state, got `{initial}`"
        )));
    }

    let scripted_inputs = ["call", "call", "call", "check"];
    let expected_actions = ["ACTION call", "ACTION call", "ACTION call", "ACTION check"];
    let expected_states = [
        ("FLOP", false),
        ("TURN", false),
        ("RIVER", false),
        ("complete", true),
    ];

    for ((input, expected_action), (expected_state, terminal)) in scripted_inputs
        .into_iter()
        .zip(expected_actions)
        .zip(expected_states)
    {
        let action = pipe_response(context.advice.surface.renderer(), input);
        if action.line() != expected_action || !action.advances_state() {
            return Err(io::Error::other(format!(
                "smoke expected `{expected_action}` after `{input}`, got `{}`",
                action.line()
            )));
        }

        let next = context.advice.surface.renderer().pipe_output();
        if terminal {
            if !next.contains("STATE complete") {
                return Err(io::Error::other(format!(
                    "smoke expected terminal state after `{input}`, got `{next}`"
                )));
            }
        } else if !next.contains(&format!("street={expected_state}")) {
            return Err(io::Error::other(format!(
                "smoke expected {expected_state} after `{input}`, got `{next}`"
            )));
        }
    }

    let mut report = format!(
        "SMOKE myosu-play ok\nadvice_source={}\ninitial_street=PREFLOP\nactions={}\nfinal_state=complete\n",
        context.advice.source,
        expected_actions.join("|"),
    );
    report.push_str(&context.discovery.smoke_lines());
    report.push_str(&context.live_query.smoke_lines());
    Ok(report)
}

fn liars_dice_smoke_report(
    context: &RenderContext,
    require_artifact: bool,
    require_discovery: bool,
    require_live_query: bool,
) -> io::Result<String> {
    ensure_artifact_selection(&context.advice, require_artifact)?;
    ensure_discovery_selection(&context.discovery, require_discovery)?;
    ensure_live_query_selection(&context.live_query, require_live_query)?;

    let initial = context.advice.surface.renderer().pipe_output();
    if !initial.contains("game=liars_dice") {
        return Err(io::Error::other(format!(
            "smoke expected liar's dice state, got `{initial}`"
        )));
    }

    let action = pipe_response(context.advice.surface.renderer(), "bid 1x4");
    if action.line() != "ACTION bid 1x4" || !action.advances_state() {
        return Err(io::Error::other(format!(
            "smoke expected liar's dice action after bid, got `{}`",
            action.line()
        )));
    }

    Ok(format!(
        "SMOKE myosu-play ok\ngame=liars_dice\nadvice_source={}\nactions=ACTION bid 1x4\nfinal_state=static_demo\n",
        context.advice.source,
    ))
}

fn ensure_artifact_selection(
    selection: &AdviceSelection,
    require_artifact: bool,
) -> io::Result<()> {
    if require_artifact && selection.source != "artifact" {
        return Err(io::Error::other(format!(
            "artifact-backed smoke required artifact advice, got {} [{} {}]",
            selection.source, selection.reason, selection.detail
        )));
    }
    Ok(())
}

fn ensure_discovery_selection(
    selection: &DiscoverySelection,
    require_discovery: bool,
) -> io::Result<()> {
    if require_discovery && selection.miner.is_none() {
        let detail = selection
            .detail
            .as_deref()
            .unwrap_or("no miner discovery detail available");
        return Err(io::Error::other(format!(
            "chain-visible miner discovery required, got {} ({detail})",
            selection.source
        )));
    }
    Ok(())
}

fn ensure_live_query_selection(
    selection: &LiveQuerySelection,
    require_live_query: bool,
) -> io::Result<()> {
    if require_live_query && selection.result.is_none() {
        let detail = selection
            .detail
            .as_deref()
            .unwrap_or("no live miner query detail available");
        return Err(io::Error::other(format!(
            "live miner query required, got {} ({detail})",
            selection.source
        )));
    }
    Ok(())
}

async fn run_train(
    game: GameSelection,
    args: AdviceArgs,
    discovery_request: DiscoveryRequest,
) -> io::Result<()> {
    let mut shell = Shell::with_screen(Screen::Lobby);
    shell.set_status(
        InteractionState::Loading,
        Some("Resolving local artifacts and miner context.".to_string()),
    );
    shell.log("Preparing play surface...".to_string());
    let loading_renderer = loading_renderer(game);
    let mut terminal = ratatui::init();
    terminal.draw(|frame| {
        let area = frame.area();
        let buf = frame.buffer_mut();
        shell.draw(area, buf, loading_renderer.as_ref());
    })?;

    let context = match resolve_context(game, args, discovery_request).await {
        Ok(context) => context,
        Err(error) => {
            ratatui::restore();
            return Err(error);
        }
    };
    shell = Shell::with_screen(startup_screen(&context.advice));
    shell.log("Type 1 to enter the NLHE demo hand.".to_string());
    shell.log("Use /quit to leave the table.".to_string());
    shell.set_status(context.startup_state(), Some(context.startup_detail()));
    for message in context.startup_messages() {
        shell.log(message);
    }
    shell.log(context.summary_line());
    shell.update_completions(context.advice.surface.renderer());
    let mut live_advice_refresh = None;
    let result = shell
        .run_terminal_with_updates(
            &mut terminal,
            context.advice.surface.renderer(),
            DEFAULT_TICK_RATE,
            |update_tx| {
                live_advice_refresh = spawn_live_advice_refresh(
                    context.advice.surface.poker_renderer_arc(),
                    context.discovery.miner.clone(),
                    Some(update_tx),
                );
            },
        )
        .await;
    ratatui::restore();
    stop_live_advice_refresh(live_advice_refresh).await;
    result
}

fn loading_renderer(game: GameSelection) -> Box<dyn GameRenderer> {
    match game {
        GameSelection::Poker => Box::new(NlheRenderer::demo()),
        GameSelection::LiarsDice => {
            Box::new(LiarsDiceRenderer::new(Some(LiarsDiceSnapshot::demo())))
        }
    }
}

async fn run_pipe(
    game: GameSelection,
    args: AdviceArgs,
    discovery_request: DiscoveryRequest,
) -> io::Result<()> {
    let context = resolve_context(game, args, discovery_request).await?;
    let pipe = PipeMode::new(context.advice.surface.renderer());

    println!("{}", context.startup_status_line());
    println!("{}", context.info_line());
    println!("{}", pipe_state_output(&context).await);
    while let Some(input) = pipe.read_input() {
        if input.is_empty() {
            continue;
        }
        let response = pipe_response(context.advice.surface.renderer(), &input);
        println!("{}", response.line());
        if response.exits_pipe() {
            break;
        }
        if response.advances_state() {
            println!("{}", pipe_state_output(&context).await);
        }
    }

    Ok(())
}

impl DiscoverySelection {
    fn not_requested() -> Self {
        Self {
            source: "not_requested",
            detail: None,
            miner: None,
        }
    }

    fn found(miner: DiscoveredMiner) -> Self {
        Self {
            source: "chain_visible",
            detail: None,
            miner: Some(miner),
        }
    }

    fn missing(detail: String) -> Self {
        Self {
            source: "chain_none",
            detail: Some(detail),
            miner: None,
        }
    }

    fn startup_issue(&self) -> Option<String> {
        if self.miner.is_some() || self.source == "not_requested" {
            return None;
        }

        let detail = self
            .detail
            .as_deref()
            .unwrap_or("no miner discovery detail available");
        Some(format!("Miner discovery unavailable ({detail})."))
    }

    fn info_fragment(&self) -> Option<String> {
        let miner = self.miner.as_ref()?;
        Some(format!(
            " miner_discovery={} discovered_miner_uid={} discovered_miner_incentive={} discovered_miner_hotkey={} discovered_miner_endpoint={}",
            self.source, miner.uid, miner.incentive, miner.hotkey, miner.endpoint
        ))
    }

    fn summary_suffix(&self) -> Option<String> {
        if let Some(miner) = &self.miner {
            return Some(format!(
                " Miner discovery: uid={} incentive={} endpoint={}.",
                miner.uid, miner.incentive, miner.endpoint
            ));
        }
        self.detail
            .as_ref()
            .map(|detail| format!(" Miner discovery: none ({detail})."))
    }

    fn smoke_lines(&self) -> String {
        if let Some(miner) = &self.miner {
            return format!(
                "miner_discovery={}\ndiscovered_miner_uid={}\ndiscovered_miner_incentive={}\ndiscovered_miner_hotkey={}\ndiscovered_miner_endpoint={}\n",
                self.source, miner.uid, miner.incentive, miner.hotkey, miner.endpoint
            );
        }
        match &self.detail {
            Some(detail) => {
                format!(
                    "miner_discovery={}\ndiscovery_detail={detail:?}\n",
                    self.source
                )
            }
            None => String::new(),
        }
    }
}

impl LiveQuerySelection {
    fn not_requested() -> Self {
        Self {
            source: "not_requested",
            detail: None,
            result: None,
        }
    }

    fn success(result: LiveMinerStrategy) -> Self {
        Self {
            source: "live_http",
            detail: None,
            result: Some(result),
        }
    }

    fn failed(detail: String) -> Self {
        Self {
            source: "live_failed",
            detail: Some(detail),
            result: None,
        }
    }

    fn startup_issue(&self) -> Option<String> {
        if self.result.is_some() || self.source == "not_requested" {
            return None;
        }

        let detail = self
            .detail
            .as_deref()
            .unwrap_or("no live miner query detail available");
        Some(format!("Live miner query unavailable ({detail})."))
    }

    fn info_fragment(&self) -> Option<String> {
        let result = self.result.as_ref()?;
        Some(format!(
            " live_miner_query={} live_miner_connect_endpoint={} live_miner_action_count={} live_miner_recommended_edge={} live_miner_recommended_action={}",
            self.source,
            result.connect_endpoint,
            result.action_count,
            result.recommended_edge,
            result.recommended_action
        ))
    }

    fn summary_suffix(&self) -> Option<String> {
        if let Some(result) = &self.result {
            return Some(format!(
                " Live miner query: ok via {} ({} actions, edge={}, action={}).",
                result.connect_endpoint,
                result.action_count,
                result.recommended_edge,
                result.recommended_action
            ));
        }
        self.detail
            .as_ref()
            .map(|detail| format!(" Live miner query: none ({detail})."))
    }

    fn smoke_lines(&self) -> String {
        if let Some(result) = &self.result {
            return format!(
                "live_miner_query={}\nlive_miner_connect_endpoint={}\nlive_miner_action_count={}\nlive_miner_recommended_edge={}\nlive_miner_recommended_action={}\n",
                self.source,
                result.connect_endpoint,
                result.action_count,
                result.recommended_edge,
                result.recommended_action
            );
        }
        match &self.detail {
            Some(detail) => {
                format!(
                    "live_miner_query={}\nlive_query_detail={detail:?}\n",
                    self.source
                )
            }
            None => String::new(),
        }
    }
}

impl RenderContext {
    fn startup_status_line(&self) -> String {
        format!(
            "STATUS startup_state={} advice_source={} selection={} origin={} reason={} \
miner_discovery={} live_query={} detail={:?}",
            startup_state_label(self.startup_state()),
            self.advice.source,
            self.advice.selection,
            self.advice.origin,
            self.advice.reason,
            self.discovery.source,
            self.live_query.source,
            self.startup_detail(),
        )
    }

    fn startup_state(&self) -> InteractionState {
        let advice_state = startup_interaction_state(&self.advice);
        if advice_state == InteractionState::Empty {
            return InteractionState::Empty;
        }

        if advice_state == InteractionState::Partial
            || self.discovery.startup_issue().is_some()
            || self.live_query.startup_issue().is_some()
        {
            return InteractionState::Partial;
        }

        advice_state
    }

    fn startup_detail(&self) -> String {
        match self.startup_state() {
            InteractionState::Empty => self.advice.detail.clone(),
            InteractionState::Partial => self
                .discovery
                .startup_issue()
                .or_else(|| self.live_query.startup_issue())
                .unwrap_or_else(|| self.advice.detail.clone()),
            InteractionState::Success => self.advice.detail.clone(),
            InteractionState::Neutral | InteractionState::Loading | InteractionState::Error => {
                self.advice.detail.clone()
            }
        }
    }

    fn startup_messages(&self) -> Vec<String> {
        let mut messages = self.advice.startup_messages();
        if let Some(issue) = self.discovery.startup_issue() {
            messages.push(issue);
        }
        if let Some(issue) = self.live_query.startup_issue() {
            messages.push(issue);
        }
        messages
    }

    fn info_line(&self) -> String {
        let mut line = self.advice.info_line();
        if let Some(fragment) = self.discovery.info_fragment() {
            line.push_str(&fragment);
        }
        if let Some(fragment) = self.live_query.info_fragment() {
            line.push_str(&fragment);
        }
        line
    }

    fn summary_line(&self) -> String {
        let mut line = self.advice.summary_line();
        if let Some(suffix) = self.discovery.summary_suffix() {
            line.push_str(&suffix);
        }
        if let Some(suffix) = self.live_query.summary_suffix() {
            line.push_str(&suffix);
        }
        line
    }

    async fn refresh_live_query(&self) -> LiveQuerySelection {
        let Some(miner) = self.discovery.miner.as_ref() else {
            return LiveQuerySelection::not_requested();
        };
        let Some(renderer) = self.advice.surface.poker_renderer() else {
            return LiveQuerySelection::not_requested();
        };
        match live::query_live_miner(miner, renderer).await {
            Ok(result) => LiveQuerySelection::success(result),
            Err(error) => LiveQuerySelection::failed(error.to_string()),
        }
    }
}

async fn resolve_context(
    game: GameSelection,
    args: AdviceArgs,
    discovery_request: DiscoveryRequest,
) -> io::Result<RenderContext> {
    let advice = demo_renderer(game, args)?;
    let discovery = resolve_discovery(discovery_request).await?;
    let live_query = resolve_live_query(advice.surface.poker_renderer(), &discovery).await?;
    if let Some(renderer) = advice.surface.poker_renderer() {
        if discovery.miner.is_some() {
            if let Some(result) = live_query.result.as_ref() {
                renderer.set_live_recommendation(&result.recommended_action);
            } else {
                renderer.set_live_advice_offline();
            }
        } else {
            renderer.clear_live_advice();
        }
    }

    Ok(RenderContext {
        advice,
        discovery,
        live_query,
    })
}

async fn resolve_discovery(request: DiscoveryRequest) -> io::Result<DiscoverySelection> {
    if !request.is_requested() {
        return Ok(DiscoverySelection::not_requested());
    }

    let chain = request.chain.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing --chain for discovery request",
        )
    })?;
    let subnet = request.subnet.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing --subnet for discovery request",
        )
    })?;
    let subnet = NetUid::from(subnet);
    let discovered = discovery::discover_best_chain_visible_miner(&chain, subnet).await?;
    Ok(match discovered {
        Some(miner) => DiscoverySelection::found(miner),
        None => DiscoverySelection::missing(format!(
            "no chain-visible miners with nonzero incentive on subnet {subnet}"
        )),
    })
}

async fn resolve_live_query(
    renderer: Option<&NlheRenderer>,
    discovery: &DiscoverySelection,
) -> io::Result<LiveQuerySelection> {
    let Some(miner) = discovery.miner.as_ref() else {
        return Ok(LiveQuerySelection::not_requested());
    };
    let Some(renderer) = renderer else {
        return Ok(LiveQuerySelection::not_requested());
    };
    if renderer.strategy_request().is_none() {
        return Ok(LiveQuerySelection::failed(
            "renderer did not expose a strategy request".to_string(),
        ));
    }

    match live::query_live_miner(miner, renderer).await {
        Ok(result) => Ok(LiveQuerySelection::success(result)),
        Err(error) => Ok(LiveQuerySelection::failed(error.to_string())),
    }
}

fn spawn_live_advice_refresh(
    renderer: Option<Arc<NlheRenderer>>,
    miner: Option<DiscoveredMiner>,
    update_tx: Option<mpsc::UnboundedSender<UpdateEvent>>,
) -> Option<LiveAdviceRefresh> {
    let renderer = renderer?;
    let miner = miner?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_task = stop.clone();
    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        let mut last_success_at: Option<Instant> = None;
        let mut reported_connectivity: Option<LiveAdviceConnectivity> = None;
        loop {
            interval.tick().await;
            if stop_task.load(Ordering::Relaxed) {
                break;
            }

            if renderer.strategy_request().is_none() {
                renderer.set_live_advice_offline();
                maybe_emit_live_advice_updates(
                    update_tx.as_ref(),
                    &mut reported_connectivity,
                    LiveAdviceConnectivity::Offline,
                    live_advice_status_detail(
                        LiveAdviceConnectivity::Offline,
                        0,
                        None,
                        Some("renderer did not expose a live strategy query"),
                    ),
                    live_advice_transition_message(
                        LiveAdviceConnectivity::Offline,
                        0,
                        None,
                        Some("renderer did not expose a live strategy query"),
                    ),
                );
                continue;
            }

            match live::query_live_miner(&miner, renderer.as_ref()).await {
                Ok(result) => {
                    last_success_at = Some(Instant::now());
                    renderer.set_live_recommendation_with_age(&result.recommended_action, 0);
                    maybe_emit_live_advice_updates(
                        update_tx.as_ref(),
                        &mut reported_connectivity,
                        LiveAdviceConnectivity::Fresh,
                        live_advice_status_detail(
                            LiveAdviceConnectivity::Fresh,
                            0,
                            Some(&result),
                            None,
                        ),
                        live_advice_transition_message(
                            LiveAdviceConnectivity::Fresh,
                            0,
                            Some(&result),
                            None,
                        ),
                    );
                }
                Err(error) => {
                    let age_seconds = last_success_at
                        .map(|instant| instant.elapsed().as_secs())
                        .unwrap_or(0);
                    renderer.mark_live_recommendation_stale_with_age(age_seconds);
                    let connectivity = if last_success_at.is_some() {
                        LiveAdviceConnectivity::Stale
                    } else {
                        LiveAdviceConnectivity::Offline
                    };
                    maybe_emit_live_advice_updates(
                        update_tx.as_ref(),
                        &mut reported_connectivity,
                        connectivity,
                        live_advice_status_detail(
                            connectivity,
                            age_seconds,
                            None,
                            Some(&error.to_string()),
                        ),
                        live_advice_transition_message(
                            connectivity,
                            age_seconds,
                            None,
                            Some(&error.to_string()),
                        ),
                    );
                }
            }
        }
    });

    Some(LiveAdviceRefresh { stop, task })
}

async fn stop_live_advice_refresh(refresh: Option<LiveAdviceRefresh>) {
    let Some(refresh) = refresh else {
        return;
    };
    refresh.stop.store(true, Ordering::Relaxed);
    let _ = refresh.task.await;
}

fn maybe_emit_live_advice_updates(
    update_tx: Option<&mpsc::UnboundedSender<UpdateEvent>>,
    reported_connectivity: &mut Option<LiveAdviceConnectivity>,
    next: LiveAdviceConnectivity,
    status_detail: String,
    message: String,
) {
    if reported_connectivity.as_ref() == Some(&next) {
        return;
    }
    *reported_connectivity = Some(next);

    let Some(update_tx) = update_tx else {
        return;
    };
    let _ = update_tx.send(UpdateEvent::Status {
        state: live_advice_interaction_state(next),
        detail: Some(status_detail),
    });
    let _ = update_tx.send(UpdateEvent::Message(message));
}

const fn live_advice_interaction_state(connectivity: LiveAdviceConnectivity) -> InteractionState {
    match connectivity {
        LiveAdviceConnectivity::Fresh => InteractionState::Success,
        LiveAdviceConnectivity::Stale => InteractionState::Partial,
        LiveAdviceConnectivity::Offline => InteractionState::Error,
    }
}

fn live_advice_status_detail(
    connectivity: LiveAdviceConnectivity,
    age_seconds: u64,
    _result: Option<&LiveMinerStrategy>,
    detail: Option<&str>,
) -> String {
    match connectivity {
        LiveAdviceConnectivity::Fresh => "Live advice connected.".to_string(),
        LiveAdviceConnectivity::Stale => match detail {
            Some(detail) => {
                format!("Live advice stale after {age_seconds}s ({detail}).")
            }
            None => format!("Live advice stale after {age_seconds}s."),
        },
        LiveAdviceConnectivity::Offline => match detail {
            Some(detail) => format!("Live advice offline ({detail})."),
            None => "Live advice offline.".to_string(),
        },
    }
}

fn live_advice_transition_message(
    connectivity: LiveAdviceConnectivity,
    age_seconds: u64,
    result: Option<&LiveMinerStrategy>,
    detail: Option<&str>,
) -> String {
    match connectivity {
        LiveAdviceConnectivity::Fresh => {
            let Some(result) = result else {
                return "Live solver advice refreshed from discovered miner.".to_string();
            };
            format!(
                "Live solver advice refreshed from {} with {}.",
                result.connect_endpoint,
                result.recommended_action.to_ascii_uppercase(),
            )
        }
        LiveAdviceConnectivity::Stale => match detail {
            Some(detail) => format!(
                "Live solver advice is stale; last successful refresh was {age_seconds}s ago ({detail})."
            ),
            None => format!(
                "Live solver advice is stale; last successful refresh was {age_seconds}s ago."
            ),
        },
        LiveAdviceConnectivity::Offline => match detail {
            Some(detail) => {
                format!(
                    "Live solver advice is offline; no successful miner refresh yet ({detail})."
                )
            }
            None => "Live solver advice is offline; no successful miner refresh yet.".to_string(),
        },
    }
}

async fn pipe_state_output(context: &RenderContext) -> String {
    let live_query = context.refresh_live_query().await;
    match context.advice.surface.poker_renderer() {
        Some(renderer) => pipe_output_for_live_query(renderer, &live_query),
        None => context.advice.surface.renderer().pipe_output(),
    }
}

fn pipe_output_for_live_query(renderer: &NlheRenderer, live_query: &LiveQuerySelection) -> String {
    let output = if let Some(result) = live_query.result.as_ref() {
        renderer.pipe_output_with_recommendation(&result.recommended_action, "live")
    } else {
        renderer.pipe_output()
    };
    append_pipe_live_query_metadata(output, live_query)
}

fn append_pipe_live_query_metadata(mut output: String, live_query: &LiveQuerySelection) -> String {
    match &live_query.result {
        Some(result) => {
            output.push_str(&format!(
                " live_query={} live_miner_advertised_endpoint={} live_miner_connect_endpoint={} live_miner_action_count={} live_miner_recommended_edge={} live_miner_recommended_action={}",
                live_query.source,
                result.advertised_endpoint,
                result.connect_endpoint,
                result.action_count,
                result.recommended_edge,
                result.recommended_action
            ));
        }
        None => {
            output.push_str(&format!(" live_query={}", live_query.source));
            if let Some(detail) = &live_query.detail {
                output.push_str(&format!(" live_query_detail={detail:?}"));
            }
        }
    }
    output
}

fn startup_state_label(state: InteractionState) -> &'static str {
    match state {
        InteractionState::Success => "success",
        InteractionState::Empty => "empty",
        InteractionState::Partial => "partial",
        InteractionState::Neutral => "neutral",
        InteractionState::Loading => "loading",
        InteractionState::Error => "error",
    }
}

fn startup_interaction_state(advice: &AdviceSelection) -> InteractionState {
    match advice.startup_state() {
        blueprint::AdviceStartupState::Success => InteractionState::Success,
        blueprint::AdviceStartupState::Empty => InteractionState::Empty,
        blueprint::AdviceStartupState::Partial => InteractionState::Partial,
    }
}

fn startup_screen(advice: &AdviceSelection) -> Screen {
    match advice.startup_state() {
        blueprint::AdviceStartupState::Empty => Screen::Onboarding,
        blueprint::AdviceStartupState::Success | blueprint::AdviceStartupState::Partial => {
            Screen::Lobby
        }
    }
}

impl PipeResponse {
    fn line(&self) -> &str {
        match self {
            Self::Action(line) | Self::Clarify(line) | Self::Error(line) | Self::Quit(line) => line,
        }
    }

    const fn advances_state(&self) -> bool {
        matches!(self, Self::Action(_))
    }

    const fn exits_pipe(&self) -> bool {
        matches!(self, Self::Quit(_))
    }
}

fn pipe_response(renderer: &dyn GameRenderer, input: &str) -> PipeResponse {
    let normalized = input.trim().to_ascii_lowercase();
    if normalized == "/quit" || normalized == "quit" {
        return PipeResponse::Quit("QUIT".to_string());
    }

    if let Some(action) = renderer.parse_input(input) {
        return PipeResponse::Action(format!("ACTION {action}"));
    }

    if let Some(clarify) = renderer.clarify(input) {
        return PipeResponse::Clarify(format!(
            "CLARIFY {clarify} legal={}",
            pipe_legal_actions(renderer).join("|")
        ));
    }

    PipeResponse::Error(format!(
        "ERROR invalid input legal={}",
        pipe_legal_actions(renderer).join("|")
    ))
}

fn pipe_legal_actions(renderer: &dyn GameRenderer) -> Vec<String> {
    let mut legal = renderer.completions();
    if !legal.iter().any(|action| action == "/quit") {
        legal.push("/quit".to_string());
    }
    legal
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_args() -> AdviceArgs {
        AdviceArgs {
            checkpoint: None,
            encoder_dir: None,
        }
    }

    const fn default_game() -> GameSelection {
        GameSelection::Poker
    }

    fn sample_discovered_miner() -> DiscoveredMiner {
        DiscoveredMiner {
            subnet: NetUid::from(7_u16),
            uid: 0,
            hotkey: "hotkey-0".to_string(),
            incentive: u16::MAX,
            endpoint: "0.0.0.0:8080".to_string(),
        }
    }

    fn sample_live_strategy() -> LiveMinerStrategy {
        LiveMinerStrategy {
            advertised_endpoint: "0.0.0.0:8080".to_string(),
            connect_endpoint: "127.0.0.1:8080".to_string(),
            action_count: 3,
            recommended_edge: "Call".to_string(),
            recommended_action: "call".to_string(),
        }
    }

    fn not_requested_context() -> RenderContext {
        RenderContext {
            advice: demo_renderer(default_game(), default_args())
                .expect("default renderer should build"),
            discovery: DiscoverySelection::not_requested(),
            live_query: LiveQuerySelection::not_requested(),
        }
    }

    fn not_requested_liars_dice_context() -> RenderContext {
        RenderContext {
            advice: demo_renderer(
                GameSelection::LiarsDice,
                AdviceArgs {
                    checkpoint: None,
                    encoder_dir: None,
                },
            )
            .expect("liar's dice demo should build"),
            discovery: DiscoverySelection::not_requested(),
            live_query: LiveQuerySelection::not_requested(),
        }
    }

    #[test]
    fn pipe_response_accepts_known_actions() {
        let fold_renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");
        let call_renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        assert_eq!(
            pipe_response(fold_renderer.surface.renderer(), "f").line(),
            "ACTION fold"
        );
        assert_eq!(
            pipe_response(call_renderer.surface.renderer(), "call").line(),
            "ACTION call"
        );
    }

    #[test]
    fn pipe_response_clarifies_ambiguous_raise() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        assert_eq!(
            pipe_response(renderer.surface.renderer(), "r").line(),
            "CLARIFY raise or bet to how much? legal=fold|call|raise 6|/quit"
        );
    }

    #[test]
    fn pipe_response_rejects_invalid_input() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        assert_eq!(
            pipe_response(renderer.surface.renderer(), "banana").line(),
            "ERROR invalid input legal=fold|call|raise 6|/quit"
        );
    }

    #[test]
    fn quit_response_exits_pipe() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        let quit = pipe_response(renderer.surface.renderer(), "/quit");
        assert_eq!(quit.line(), "QUIT");
        assert!(quit.exits_pipe());
    }

    #[test]
    fn pipe_legal_actions_always_include_quit() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        assert_eq!(
            pipe_legal_actions(renderer.surface.renderer()),
            vec!["fold", "call", "raise 6", "/quit"]
        );
    }

    #[test]
    fn demo_renderer_starts_with_preflop_state() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");

        assert!(
            renderer
                .surface
                .renderer()
                .pipe_output()
                .contains("street=PREFLOP")
        );
        assert!(
            renderer
                .surface
                .renderer()
                .pipe_output()
                .contains("actions=fold|call|raise 6|/quit")
        );
    }

    #[test]
    fn smoke_report_proves_complete_hand_progression() {
        let context = not_requested_context();
        let report =
            smoke_report(&context, false, false, false).expect("smoke report should succeed");

        assert!(report.contains("SMOKE myosu-play ok"));
        assert!(report.contains("initial_street=PREFLOP"));
        assert!(report.contains("actions=ACTION call|ACTION call|ACTION call|ACTION check"));
        assert!(report.contains("final_state=complete"));
    }

    #[test]
    fn liars_dice_smoke_report_proves_local_surface_progression() {
        let context = not_requested_liars_dice_context();
        let report =
            smoke_report(&context, false, false, false).expect("smoke report should succeed");

        assert!(report.contains("SMOKE myosu-play ok"));
        assert!(report.contains("game=liars_dice"));
        assert!(report.contains("actions=ACTION bid 1x4"));
        assert!(report.contains("final_state=static_demo"));
    }

    #[test]
    fn smoke_report_rejects_generated_advice_when_artifact_required() {
        let selection = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "none",
            reason: "missing",
            detail: "no local blueprint artifacts found".to_string(),
        };
        let error = ensure_artifact_selection(&selection, true)
            .expect_err("artifact-required smoke should fail without artifacts");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(
            error
                .to_string()
                .contains("artifact-backed smoke required artifact advice")
        );
    }

    #[test]
    fn smoke_report_rejects_missing_discovery_when_required() {
        let context = not_requested_context();
        let error = smoke_report(&context, false, true, false)
            .expect_err("discovery-required smoke should fail without miner discovery");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(
            error
                .to_string()
                .contains("chain-visible miner discovery required")
        );
    }

    #[test]
    fn smoke_report_rejects_missing_live_query_when_required() {
        let context = RenderContext {
            advice: demo_renderer(default_game(), default_args())
                .expect("default renderer should build"),
            discovery: DiscoverySelection::found(sample_discovered_miner()),
            live_query: LiveQuerySelection::failed("miner offline".to_string()),
        };
        let error = smoke_report(&context, false, false, true)
            .expect_err("live-query-required smoke should fail without live query");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(error.to_string().contains("live miner query required"));
    }

    #[test]
    fn smoke_report_includes_chain_visible_discovery_when_requested() {
        let context = RenderContext {
            advice: demo_renderer(default_game(), default_args())
                .expect("default renderer should build"),
            discovery: DiscoverySelection::found(sample_discovered_miner()),
            live_query: LiveQuerySelection::success(sample_live_strategy()),
        };
        let report = smoke_report(&context, false, true, true)
            .expect("smoke report should include discovery");

        assert!(report.contains("miner_discovery=chain_visible"));
        assert!(report.contains("discovered_miner_uid=0"));
        assert!(report.contains("live_miner_query=live_http"));
        assert!(report.contains("live_miner_connect_endpoint=127.0.0.1:8080"));
        assert!(report.contains("live_miner_recommended_action=call"));
    }

    #[test]
    fn startup_interaction_state_maps_blueprint_states() {
        let success = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "artifact",
            selection: "auto",
            origin: "home",
            reason: "auto_loaded",
            detail: "loaded".to_string(),
        };
        let empty = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "none",
            reason: "missing",
            detail: "no local blueprint artifacts found".to_string(),
        };
        let partial = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "repo",
            reason: "load_failed",
            detail: "checkpoint decode failed".to_string(),
        };

        assert_eq!(
            startup_interaction_state(&success),
            InteractionState::Success
        );
        assert_eq!(startup_interaction_state(&empty), InteractionState::Empty);
        assert_eq!(
            startup_interaction_state(&partial),
            InteractionState::Partial
        );
    }

    #[test]
    fn startup_screen_maps_empty_state_to_onboarding() {
        let empty = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "none",
            reason: "missing",
            detail: "no local blueprint artifacts found".to_string(),
        };
        let partial = AdviceSelection {
            game: GameSelection::Poker,
            surface: blueprint::AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "repo",
            reason: "load_failed",
            detail: "checkpoint decode failed".to_string(),
        };

        assert_eq!(startup_screen(&empty), Screen::Onboarding);
        assert_eq!(startup_screen(&partial), Screen::Lobby);
    }

    #[test]
    fn startup_state_becomes_partial_when_discovery_returns_zero_results() {
        let context = RenderContext {
            advice: demo_renderer(default_game(), default_args())
                .expect("default renderer should build"),
            discovery: DiscoverySelection::missing(
                "no chain-visible miners with nonzero incentive on subnet 7".to_string(),
            ),
            live_query: LiveQuerySelection::not_requested(),
        };

        assert_eq!(context.startup_state(), InteractionState::Partial);
        assert!(
            context
                .startup_detail()
                .contains("Miner discovery unavailable"),
            "startup detail should mention discovery failure"
        );
        assert!(
            context
                .startup_status_line()
                .contains("startup_state=partial")
        );
        assert!(
            context
                .startup_status_line()
                .contains("miner_discovery=chain_none")
        );
    }

    #[test]
    fn startup_state_becomes_partial_when_live_query_fails() {
        let context = RenderContext {
            advice: demo_renderer(default_game(), default_args())
                .expect("default renderer should build"),
            discovery: DiscoverySelection::found(sample_discovered_miner()),
            live_query: LiveQuerySelection::failed("timed out connecting to miner".to_string()),
        };

        assert_eq!(context.startup_state(), InteractionState::Partial);
        assert!(
            context
                .startup_detail()
                .contains("Live miner query unavailable"),
            "startup detail should mention live query failure"
        );
        assert!(
            context
                .startup_status_line()
                .contains("live_query=live_failed")
        );
    }

    #[test]
    fn pipe_output_carries_live_query_success_metadata() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");
        let output = pipe_output_for_live_query(
            renderer
                .surface
                .poker_renderer()
                .expect("poker renderer should exist"),
            &LiveQuerySelection::success(sample_live_strategy()),
        );

        assert!(output.contains("advisor=live"));
        assert!(output.contains("live_query=live_http"));
        assert!(output.contains("live_miner_advertised_endpoint=0.0.0.0:8080"));
        assert!(output.contains("live_miner_connect_endpoint=127.0.0.1:8080"));
        assert!(output.contains("live_miner_action_count=3"));
        assert!(output.contains("live_miner_recommended_edge=Call"));
        assert!(output.contains("live_miner_recommended_action=call"));
    }

    #[test]
    fn pipe_output_carries_live_query_failure_metadata() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");
        let output = pipe_output_for_live_query(
            renderer
                .surface
                .poker_renderer()
                .expect("poker renderer should exist"),
            &LiveQuerySelection::failed("timed out connecting to miner".to_string()),
        );

        assert!(output.contains("recommend="));
        assert!(output.contains("live_query=live_failed"));
        assert!(output.contains("live_query_detail=\"timed out connecting to miner\""));
    }

    #[test]
    fn pipe_output_skips_live_query_metadata_when_not_requested() {
        let renderer =
            demo_renderer(default_game(), default_args()).expect("default renderer should build");
        let output = pipe_output_for_live_query(
            renderer
                .surface
                .poker_renderer()
                .expect("poker renderer should exist"),
            &LiveQuerySelection::not_requested(),
        );

        assert!(output.contains("recommend="));
        assert!(output.contains("live_query=not_requested"));
    }

    #[test]
    fn live_advice_transition_messages_match_connectivity_state() {
        assert_eq!(
            live_advice_transition_message(
                LiveAdviceConnectivity::Fresh,
                0,
                Some(&sample_live_strategy()),
                None,
            ),
            "Live solver advice refreshed from 127.0.0.1:8080 with CALL.".to_string()
        );
        assert_eq!(
            live_advice_transition_message(
                LiveAdviceConnectivity::Stale,
                7,
                None,
                Some("timed out connecting to miner"),
            ),
            "Live solver advice is stale; last successful refresh was 7s ago (timed out connecting to miner).".to_string()
        );
        assert_eq!(
            live_advice_transition_message(
                LiveAdviceConnectivity::Offline,
                0,
                None,
                Some("failed to connect to miner"),
            ),
            "Live solver advice is offline; no successful miner refresh yet (failed to connect to miner).".to_string()
        );
    }

    #[test]
    fn live_advice_messages_only_emit_on_connectivity_transitions() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut reported = None;

        maybe_emit_live_advice_updates(
            Some(&tx),
            &mut reported,
            LiveAdviceConnectivity::Offline,
            live_advice_status_detail(
                LiveAdviceConnectivity::Offline,
                0,
                None,
                Some("failed to connect to miner"),
            ),
            live_advice_transition_message(
                LiveAdviceConnectivity::Offline,
                0,
                None,
                Some("failed to connect to miner"),
            ),
        );
        maybe_emit_live_advice_updates(
            Some(&tx),
            &mut reported,
            LiveAdviceConnectivity::Offline,
            live_advice_status_detail(
                LiveAdviceConnectivity::Offline,
                0,
                None,
                Some("failed to connect to miner"),
            ),
            live_advice_transition_message(
                LiveAdviceConnectivity::Offline,
                0,
                None,
                Some("failed to connect to miner"),
            ),
        );
        maybe_emit_live_advice_updates(
            Some(&tx),
            &mut reported,
            LiveAdviceConnectivity::Fresh,
            live_advice_status_detail(
                LiveAdviceConnectivity::Fresh,
                0,
                Some(&sample_live_strategy()),
                None,
            ),
            live_advice_transition_message(
                LiveAdviceConnectivity::Fresh,
                0,
                Some(&sample_live_strategy()),
                None,
            ),
        );

        let first_status = rx.try_recv().expect("offline status should emit");
        let first_message = rx.try_recv().expect("offline transition should emit");
        let second_status = rx.try_recv().expect("fresh status should emit");
        let second_message = rx.try_recv().expect("fresh transition should emit");
        assert!(
            rx.try_recv().is_err(),
            "duplicate connectivity state should not emit"
        );

        let UpdateEvent::Status {
            state: first_state,
            detail: Some(first_detail),
        } = first_status
        else {
            panic!("expected status update for offline transition");
        };
        assert_eq!(first_state, InteractionState::Error);
        assert_eq!(
            first_detail,
            "Live advice offline (failed to connect to miner)."
        );

        let UpdateEvent::Message(first_message) = first_message else {
            panic!("expected transcript message for offline transition");
        };
        assert_eq!(
            first_message,
            "Live solver advice is offline; no successful miner refresh yet (failed to connect to miner)."
        );

        let UpdateEvent::Status {
            state: second_state,
            detail: Some(second_detail),
        } = second_status
        else {
            panic!("expected status update for fresh transition");
        };
        assert_eq!(second_state, InteractionState::Success);
        assert_eq!(second_detail, "Live advice connected.");

        let UpdateEvent::Message(second_message) = second_message else {
            panic!("expected transcript message for fresh transition");
        };
        assert_eq!(
            second_message,
            "Live solver advice refreshed from 127.0.0.1:8080 with CALL."
        );
    }

    #[test]
    fn live_advice_interaction_state_maps_connectivity() {
        assert_eq!(
            live_advice_interaction_state(LiveAdviceConnectivity::Fresh),
            InteractionState::Success
        );
        assert_eq!(
            live_advice_interaction_state(LiveAdviceConnectivity::Stale),
            InteractionState::Partial
        );
        assert_eq!(
            live_advice_interaction_state(LiveAdviceConnectivity::Offline),
            InteractionState::Error
        );
    }

    #[test]
    fn loading_renderer_matches_selected_game_surface() {
        assert!(
            loading_renderer(GameSelection::Poker)
                .pipe_output()
                .contains("street=PREFLOP")
        );
        assert!(
            loading_renderer(GameSelection::LiarsDice)
                .pipe_output()
                .contains("game=liars_dice")
        );
    }

    #[test]
    fn discovery_request_requires_chain_pairing() {
        let cli = Cli::parse_from([
            "myosu-play",
            "--smoke-test",
            "--chain",
            "ws://127.0.0.1:9944",
        ]);
        let error = DiscoveryRequest::from_cli(&cli).expect_err("chain-only discovery should fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
