#![allow(dead_code)]

use crate::blueprint::BlueprintBackend;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use myosu_games_poker::NlheRenderer;
use myosu_games_poker::renderer::NlheState;
use rand::distributions::{Distribution, WeightedIndex};
use rbp_cards::{Board, Card, Deck, Hand, Hole, Street, Strength};
use rbp_core::Chips;
use rbp_gameplay::{Action, Game, PositionName, Recall, Turn};

const DEFAULT_PRACTICE_CHIPS: i32 = 10_000;
const DEFAULT_BOT_DELAY_MS: u64 = 300;

#[derive(Debug, Clone, Default)]
pub struct PendingTrainingCommands {
    hero_hole: Option<Hole>,
    board_script: Option<Vec<Card>>,
}

#[derive(Debug, Clone)]
pub struct HandResult {
    pub hand_num: u32,
    pub hero_delta: Chips,
    pub showdown: bool,
    pub result_text: String,
}

#[derive(Debug, Clone)]
struct SessionRecall {
    root: Game,
    actions: Vec<Action>,
}

impl Recall for SessionRecall {
    fn root(&self) -> Game {
        self.root
    }

    fn actions(&self) -> &[Action] {
        &self.actions
    }
}

pub trait BotBackend: Send + Sync + std::fmt::Debug {
    fn strategy_name(&self) -> &str;

    fn action_distribution(&self, recall: &dyn Recall, seat: usize) -> Vec<(Action, f64)>;

    fn select_action(&self, recall: &dyn Recall, seat: usize) -> Action {
        sample_from_distribution(&self.action_distribution(recall, seat))
    }
}

#[derive(Debug, Default)]
pub struct HeuristicBackend;

impl BotBackend for HeuristicBackend {
    fn strategy_name(&self) -> &str {
        "heuristic"
    }

    fn action_distribution(&self, recall: &dyn Recall, seat: usize) -> Vec<(Action, f64)> {
        let game = recall.head();
        let legal = game.legal();
        if legal.is_empty() {
            return Vec::new();
        }

        let hand_score = if game.board().street() == Street::Pref {
            preflop_score(game.seats()[seat].cards())
        } else {
            postflop_score(game, seat)
        };

        let pot = f64::from(game.pot().max(1));
        let to_call = f64::from(game.to_call().max(0));
        let pressure = (to_call / pot).min(1.0);

        let mut weighted = legal
            .into_iter()
            .map(|action| {
                let weight = match action {
                    Action::Check => 0.45 + hand_score * 0.35,
                    Action::Call(_) => 0.15 + hand_score * 0.65 + (1.0 - pressure) * 0.20,
                    Action::Raise(_) => 0.05 + hand_score * 0.95,
                    Action::Shove(_) => {
                        if hand_score > 0.80 {
                            0.15 + hand_score * 0.85
                        } else {
                            0.02 + hand_score * 0.10
                        }
                    }
                    Action::Fold => 0.10 + (1.0 - hand_score) * 0.95 + pressure * 0.35,
                    Action::Blind(_) | Action::Draw(_) => 1.0,
                };
                (action, weight.max(0.01))
            })
            .collect::<Vec<_>>();

        normalize_distribution(&mut weighted);
        weighted
    }
}

#[derive(Debug)]
pub struct TrainingTable {
    hand_root: Game,
    game: Game,
    history: Vec<Action>,
    hand_num: u32,
    practice_chips: i32,
    hero_seat: usize,
    pending: PendingTrainingCommands,
    active_board_script: Vec<Card>,
    force_showdown: bool,
    bot_backend: Arc<dyn BotBackend>,
    bot_delay_ms: u64,
    strategy_status: String,
    last_result: Option<HandResult>,
}

impl TrainingTable {
    pub fn new() -> Self {
        Self::with_backend_and_delay(
            Arc::new(HeuristicBackend),
            bot_delay_from_env(DEFAULT_BOT_DELAY_MS),
        )
    }

    pub fn with_backend(bot_backend: Arc<dyn BotBackend>) -> Self {
        Self::with_backend_and_delay(bot_backend, bot_delay_from_env(DEFAULT_BOT_DELAY_MS))
    }

    pub fn with_backend_and_delay(bot_backend: Arc<dyn BotBackend>, bot_delay_ms: u64) -> Self {
        let game = Game::root();
        let strategy_status = format!("bot strategy: {}", bot_backend.strategy_name());
        Self {
            hand_root: game,
            game,
            history: Vec::new(),
            hand_num: 1,
            practice_chips: DEFAULT_PRACTICE_CHIPS,
            hero_seat: 0,
            pending: PendingTrainingCommands::default(),
            active_board_script: Vec::new(),
            force_showdown: false,
            bot_backend,
            bot_delay_ms,
            strategy_status,
            last_result: None,
        }
    }

    pub fn with_fallback_reason(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let mut table = Self::with_backend(Arc::new(HeuristicBackend));
        table.strategy_status = format!("bot strategy: heuristic · {reason}");
        table
    }

    pub fn hand_num(&self) -> u32 {
        self.hand_num
    }

    pub fn practice_chips(&self) -> i32 {
        self.practice_chips
    }

    pub fn strategy_status(&self) -> &str {
        &self.strategy_status
    }

    pub fn set_strategy_status(&mut self, strategy_status: impl Into<String>) {
        self.strategy_status = strategy_status.into();
    }

    pub fn bot_delay_ms(&self) -> u64 {
        self.bot_delay_ms
    }

    pub fn hero_seat(&self) -> usize {
        self.hero_seat
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn history(&self) -> &[Action] {
        &self.history
    }

    pub fn last_result(&self) -> Option<&HandResult> {
        self.last_result.as_ref()
    }

    pub fn is_terminal(&self) -> bool {
        self.game.turn() == Turn::Terminal
    }

    pub fn hero_position_label(&self) -> String {
        let dealer = self.game.dealer().position();
        PositionName::from_seat(self.hero_seat, dealer, self.game.n()).to_string()
    }

    pub fn bot_position_label(&self) -> String {
        let dealer = self.game.dealer().position();
        PositionName::from_seat(self.bot_seat(), dealer, self.game.n()).to_string()
    }

    pub fn renderer(&self) -> NlheRenderer {
        let hero = self.game.seats()[self.hero_seat];
        let bot = self.game.seats()[self.bot_seat()];
        let board = board_cards(self.game.board());
        let hand_num = self.hand_num;
        let to_call = self.game.to_call().max(0) as u32;
        let pot = self.game.pot().max(0) as u32;
        let hero_stack = hero.stack().max(0) as u32;
        let bot_stack = bot.stack().max(0) as u32;
        let hero_position = self.hero_position_label();
        let bot_position = self.bot_position_label();

        if self.is_terminal() {
            let result = self
                .last_result
                .as_ref()
                .map(|result| result.result_text.clone())
                .unwrap_or_else(|| "hand complete".to_string());
            return NlheRenderer::new(NlheState::Showdown {
                hand_num,
                hero_hole: hole_text(hero.cards()),
                opponent_hole: hole_text(bot.cards()),
                board: board_array_five(&board),
                pot,
                result,
            });
        }

        match self.game.turn() {
            Turn::Choice(seat) if seat == self.hero_seat => {
                if self.game.board().street() == Street::Pref {
                    NlheRenderer::new(NlheState::Preflop {
                        hero_hole: hole_text(hero.cards()),
                        hero_stack,
                        hero_position: boxed_str(hero_position),
                        opponent_stack: bot_stack,
                        opponent_position: boxed_str(bot_position),
                        pot,
                        to_call,
                        hand_num,
                        has_decision: true,
                    })
                } else {
                    NlheRenderer::new(NlheState::Flop {
                        hero_hole: hole_text(hero.cards()),
                        hero_stack,
                        hero_position: boxed_str(hero_position),
                        opponent_stack: bot_stack,
                        opponent_position: boxed_str(bot_position),
                        pot,
                        to_call,
                        board: board_array_three(&board),
                        hand_num,
                        has_decision: true,
                    })
                }
            }
            _ => {
                if self.game.board().street() == Street::Pref {
                    NlheRenderer::new(NlheState::Preflop {
                        hero_hole: hole_text(hero.cards()),
                        hero_stack,
                        hero_position: boxed_str(hero_position),
                        opponent_stack: bot_stack,
                        opponent_position: boxed_str(bot_position),
                        pot,
                        to_call,
                        hand_num,
                        has_decision: false,
                    })
                } else {
                    NlheRenderer::new(NlheState::Flop {
                        hero_hole: hole_text(hero.cards()),
                        hero_stack,
                        hero_position: boxed_str(hero_position),
                        opponent_stack: bot_stack,
                        opponent_position: boxed_str(bot_position),
                        pot,
                        to_call,
                        board: board_array_three(&board),
                        hand_num,
                        has_decision: false,
                    })
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<Option<String>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        if trimmed.starts_with('/') {
            return self.handle_command(trimmed);
        }

        let action = self.parse_hero_action(trimmed)?;
        self.apply_hero_action(action)?;
        Ok(Some(format!("hero {}", action_summary(action))))
    }

    pub fn handle_command(&mut self, command: &str) -> Result<Option<String>> {
        let mut parts = command.split_whitespace();
        let name = parts.next().unwrap_or_default();

        match name {
            "/deal" => {
                let cards = parts.collect::<Vec<_>>().join(" ");
                let hole = parse_hole(&cards)?;
                if let Some(board) = self.pending.board_script.as_ref() {
                    ensure_disjoint_hole_and_board(hole, board)?;
                }
                self.pending.hero_hole = Some(hole);
                Ok(Some(format!(
                    "next hand hero hole cards set to {}",
                    hole_text(hole)
                )))
            }
            "/board" => {
                let cards = parse_board_cards(parts.collect::<Vec<_>>().join(" ").as_str())?;
                if let Some(hole) = self.pending.hero_hole {
                    ensure_disjoint_hole_and_board(hole, &cards)?;
                }
                self.pending.board_script = Some(cards.clone());
                Ok(Some(format!(
                    "next hand board script set to {}",
                    cards_text(&cards)
                )))
            }
            "/stack" | "/bot-stack" => {
                let amount = parts
                    .next()
                    .context("stack command requires a big blind amount")?;
                let _parsed = amount
                    .parse::<i32>()
                    .with_context(|| format!("invalid stack amount: {amount}"))?;
                bail!("stack overrides are blocked until rbp-gameplay exposes public stack setters")
            }
            "/showdown" => {
                self.force_showdown = true;
                self.advance_until_hero_or_terminal_sync()?;
                Ok(Some("forcing passive runout to showdown".to_string()))
            }
            other => bail!("unknown training command: {other}"),
        }
    }

    pub fn parse_hero_action(&self, input: &str) -> Result<Action> {
        if self.game.turn() != Turn::Choice(self.hero_seat) {
            bail!("hero cannot act when it is not their turn");
        }

        let input = input.trim().to_lowercase();
        let action = match input.as_str() {
            "f" | "fold" => Action::Fold,
            "x" | "check" => {
                if self.game.may_check() {
                    Action::Check
                } else if self.game.may_call() {
                    self.game.calls()
                } else if self.game.may_shove() {
                    self.game.shove()
                } else {
                    Action::Fold
                }
            }
            "c" | "call" => {
                if self.game.may_check() {
                    Action::Check
                } else if self.game.may_call() {
                    self.game.calls()
                } else if self.game.may_shove() {
                    self.game.shove()
                } else {
                    bail!("call is not legal in the current state");
                }
            }
            "r" | "raise" => {
                if self.game.may_raise() {
                    self.game.raise()
                } else if self.game.may_shove() {
                    self.game.shove()
                } else {
                    bail!("raise is not legal in the current state");
                }
            }
            "s" | "shove" | "all-in" => {
                if self.game.may_shove() {
                    self.game.shove()
                } else {
                    bail!("shove is not legal in the current state");
                }
            }
            other => {
                if let Some(rest) = other.strip_prefix("r ") {
                    parse_raise_amount(rest, &self.game)?
                } else if let Ok(amount) = other.parse::<Chips>() {
                    amount_to_raise_action(amount, &self.game)?
                } else {
                    bail!("could not parse hero action: {other}");
                }
            }
        };

        if self.game.is_allowed(&action) {
            Ok(action)
        } else {
            bail!("illegal action: {}", action_summary(action))
        }
    }

    pub fn apply_hero_action(&mut self, action: Action) -> Result<()> {
        if self.game.turn() != Turn::Choice(self.hero_seat) {
            bail!("hero cannot act when it is not their turn");
        }
        self.apply_action(action)?;
        self.advance_until_hero_or_terminal_sync()
    }

    pub fn start_next_hand(&mut self) -> Result<()> {
        if !self.is_terminal() {
            bail!("cannot start the next hand before the current hand is terminal");
        }

        let next = self.game.continuation().unwrap_or_else(Game::root);
        let prepared = self.prepare_new_hand(next)?;
        self.hand_num += 1;
        self.hand_root = prepared;
        self.game = prepared;
        self.history.clear();
        self.last_result = None;
        self.force_showdown = false;
        Ok(())
    }

    pub fn advance_until_hero_or_terminal_sync(&mut self) -> Result<()> {
        loop {
            match self.game.turn() {
                Turn::Terminal => {
                    self.finalize_terminal_hand();
                    return Ok(());
                }
                Turn::Chance => {
                    let draw = self.next_draw_action()?;
                    self.apply_action(draw)?;
                }
                Turn::Choice(seat) if seat == self.hero_seat && !self.force_showdown => {
                    return Ok(());
                }
                Turn::Choice(seat) => {
                    let action = if self.force_showdown {
                        passive_showdown_action(&self.game)
                    } else {
                        let recall = self.current_recall();
                        let action = self.bot_backend.select_action(&recall, seat);
                        sanitize_action(&self.game, action)
                    };
                    self.apply_action(action)?;
                }
            }
        }
    }

    pub async fn advance_until_hero_or_terminal(&mut self) -> Result<()> {
        loop {
            match self.game.turn() {
                Turn::Terminal => {
                    self.finalize_terminal_hand();
                    return Ok(());
                }
                Turn::Chance => {
                    let draw = self.next_draw_action()?;
                    self.apply_action(draw)?;
                }
                Turn::Choice(seat) if seat == self.hero_seat && !self.force_showdown => {
                    return Ok(());
                }
                Turn::Choice(seat) => {
                    if seat == self.bot_seat() && self.bot_delay_ms > 0 {
                        tokio::time::sleep(Duration::from_millis(self.bot_delay_ms)).await;
                    }

                    let action = if self.force_showdown {
                        passive_showdown_action(&self.game)
                    } else {
                        let recall = self.current_recall();
                        let action = self.bot_backend.select_action(&recall, seat);
                        sanitize_action(&self.game, action)
                    };
                    self.apply_action(action)?;
                }
            }
        }
    }

    fn prepare_new_hand(&mut self, game: Game) -> Result<Game> {
        let board_script = self.pending.board_script.take().unwrap_or_default();
        let hero_hole = self.pending.hero_hole.take();

        let prepared = if let Some(hero_hole) = hero_hole {
            let villain = sample_villain_hole(hero_hole, &board_script)?;
            game.wipe(hero_hole)
                .assume(Turn::Choice(self.hero_seat), villain)
        } else {
            game
        };

        self.active_board_script = board_script;
        Ok(prepared)
    }

    fn bot_seat(&self) -> usize {
        (self.hero_seat + 1) % self.game.n()
    }

    fn current_recall(&self) -> SessionRecall {
        SessionRecall {
            root: self.hand_root,
            actions: self.history.clone(),
        }
    }

    fn next_draw_action(&self) -> Result<Action> {
        let street = self.game.street();
        let start = match street {
            Street::Pref => 0,
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
        };
        let needed = street.next().n_revealed();

        if self.active_board_script.len() >= start + needed {
            let slice = self.active_board_script[start..start + needed].to_vec();
            let action = Action::Draw(Hand::from(slice));
            if self.game.is_allowed(&action) {
                return Ok(action);
            }
            bail!(
                "board script produced an illegal draw for {}",
                street.label()
            );
        }

        Ok(self.game.reveal())
    }

    fn apply_action(&mut self, action: Action) -> Result<()> {
        self.game = self.game.try_apply(action)?;
        self.history.push(action);
        if self.game.turn() == Turn::Terminal {
            self.finalize_terminal_hand();
        }
        Ok(())
    }

    fn finalize_terminal_hand(&mut self) {
        if self.last_result.is_some() {
            return;
        }

        let settlements = self.game.settlements();
        let hero_delta = settlements[self.hero_seat].won();
        self.practice_chips += i32::from(hero_delta);
        let showdown = self.game.is_showdown();
        let result_text = if hero_delta >= 0 {
            format!("you win {}bb", hero_delta)
        } else {
            format!("solver wins {}bb", hero_delta.abs())
        };
        self.last_result = Some(HandResult {
            hand_num: self.hand_num,
            hero_delta,
            showdown,
            result_text,
        });
    }
}

impl Default for TrainingTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resolve_training_backend() -> (Arc<dyn BotBackend>, String) {
    match BlueprintBackend::load_default() {
        Ok(backend) => {
            let strategy_status = backend.strategy_status();
            (Arc::new(backend), strategy_status)
        }
        Err(error) => (
            Arc::new(HeuristicBackend),
            format!("bot strategy: heuristic · {}", error.fallback_reason()),
        ),
    }
}

pub fn bot_delay_from_env(default_ms: u64) -> u64 {
    env::var("MYOSU_BOT_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default_ms)
}

fn sample_from_distribution(distribution: &[(Action, f64)]) -> Action {
    match distribution.len() {
        0 => Action::Check,
        1 => distribution[0].0,
        _ => {
            let weights = distribution
                .iter()
                .map(|(_, weight)| weight.max(0.0))
                .collect::<Vec<_>>();
            match WeightedIndex::new(&weights) {
                Ok(index) => {
                    let mut rng = rand::thread_rng();
                    distribution[index.sample(&mut rng)].0
                }
                Err(_) => distribution[0].0,
            }
        }
    }
}

fn normalize_distribution(weighted: &mut Vec<(Action, f64)>) {
    let total = weighted.iter().map(|(_, weight)| *weight).sum::<f64>();
    if total <= f64::EPSILON {
        let even = 1.0 / weighted.len() as f64;
        for (_, weight) in weighted.iter_mut() {
            *weight = even;
        }
        return;
    }

    for (_, weight) in weighted.iter_mut() {
        *weight /= total;
    }
}

fn preflop_score(hole: Hole) -> f64 {
    let cards = Vec::<Card>::from(Hand::from(hole));
    let a = cards[0];
    let b = cards[1];
    let ra = f64::from(u8::from(a.rank()) + 2);
    let rb = f64::from(u8::from(b.rank()) + 2);
    let high = ra.max(rb);
    let low = ra.min(rb);
    let pair = a.rank() == b.rank();
    let suited = a.suit() == b.suit();
    let gap = (high - low) as i32;

    let mut score = (high + low) / 28.0;
    if pair {
        score += 0.35;
    }
    if suited {
        score += 0.10;
    }
    if gap <= 1 {
        score += 0.07;
    } else if gap == 2 {
        score += 0.03;
    }
    if high >= 13.0 {
        score += 0.08;
    }

    score.min(1.0)
}

fn postflop_score(game: Game, seat: usize) -> f64 {
    let hand = Hand::from(game.board()) + Hand::from(game.seats()[seat].cards());
    let strength_label = format!("{}", Strength::from(hand));
    if strength_label.starts_with("StraightFlush") || strength_label.starts_with("FourOfAKind") {
        0.98
    } else if strength_label.starts_with("Flush") || strength_label.starts_with("FullHouse") {
        0.90
    } else if strength_label.starts_with("Straight") || strength_label.starts_with("ThreeOfAKind") {
        0.78
    } else if strength_label.starts_with("TwoPair") {
        0.68
    } else if strength_label.starts_with("OnePair") {
        0.52
    } else {
        0.26
    }
}

fn passive_showdown_action(game: &Game) -> Action {
    if game.may_check() {
        Action::Check
    } else if game.may_call() {
        game.calls()
    } else if game.may_shove() {
        game.shove()
    } else {
        Action::Fold
    }
}

fn sanitize_action(game: &Game, action: Action) -> Action {
    if game.is_allowed(&action) {
        return action;
    }

    if game.may_check() {
        Action::Check
    } else if game.may_call() {
        game.calls()
    } else if game.may_shove() {
        game.shove()
    } else if game.may_fold() {
        Action::Fold
    } else {
        game.legal().first().copied().unwrap_or(Action::Check)
    }
}

fn parse_raise_amount(amount: &str, game: &Game) -> Result<Action> {
    let amount = amount
        .trim()
        .parse::<Chips>()
        .with_context(|| format!("invalid raise amount: {amount}"))?;
    amount_to_raise_action(amount, game)
}

fn amount_to_raise_action(amount: Chips, game: &Game) -> Result<Action> {
    let action = if amount >= game.to_shove() {
        game.shove()
    } else {
        Action::Raise(amount)
    };

    if game.is_allowed(&action) {
        Ok(action)
    } else {
        bail!("raise amount {amount} is outside the legal range")
    }
}

fn parse_board_cards(input: &str) -> Result<Vec<Card>> {
    let cards = Card::parse(&normalize_card_notation(input)).map_err(|err| anyhow!(err))?;
    match cards.len() {
        3..=5 => {}
        n => bail!("board script must contain 3, 4, or 5 cards; got {n}"),
    }

    let hand = Hand::from(cards.clone());
    if hand.size() != cards.len() {
        bail!("board script contains duplicate cards");
    }

    Ok(cards)
}

fn ensure_disjoint_hole_and_board(hole: Hole, board: &[Card]) -> Result<()> {
    let hero = Hand::from(hole);
    let board_hand = Hand::from(board.to_vec());
    if Hand::overlaps(&hero, &board_hand) {
        bail!("hero hole cards overlap the requested board script");
    }
    Ok(())
}

fn parse_hole(input: &str) -> Result<Hole> {
    Hole::try_from(normalize_card_notation(input).as_str()).map_err(|err| anyhow!(err))
}

fn normalize_card_notation(input: &str) -> String {
    input
        .replace('♠', "s")
        .replace('♥', "h")
        .replace('♦', "d")
        .replace('♣', "c")
}

fn sample_villain_hole(hero_hole: Hole, board: &[Card]) -> Result<Hole> {
    let used = Hand::from(hero_hole) + Hand::from(board.to_vec());
    let mut deck = Deck::from(used.complement());
    let first = deck.draw();
    let second = deck.draw();
    if first == second {
        bail!("failed to sample a distinct villain hole");
    }
    Ok(Hole::from((first, second)))
}

fn hole_text(hole: Hole) -> String {
    cards_text(&Vec::<Card>::from(Hand::from(hole)))
}

fn board_cards(board: Board) -> Vec<Card> {
    Vec::<Card>::from(Hand::from(board))
}

fn cards_text(cards: &[Card]) -> String {
    cards.iter().map(card_text).collect::<Vec<_>>().join(" ")
}

fn card_text(card: &Card) -> String {
    format!("{}{}", card.rank(), card.suit().ascii())
}

fn board_array_three(cards: &[Card]) -> [String; 3] {
    let mut values = ["·".to_string(), "·".to_string(), "·".to_string()];
    for (index, card) in cards.iter().take(3).enumerate() {
        values[index] = card_text(card);
    }
    values
}

fn board_array_five(cards: &[Card]) -> [String; 5] {
    let mut values = [
        "·".to_string(),
        "·".to_string(),
        "·".to_string(),
        "·".to_string(),
        "·".to_string(),
    ];
    for (index, card) in cards.iter().take(5).enumerate() {
        values[index] = card_text(card);
    }
    values
}

fn boxed_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn action_summary(action: Action) -> String {
    match action {
        Action::Fold => "folds".to_string(),
        Action::Check => "checks".to_string(),
        Action::Call(amount) => format!("calls {amount}bb"),
        Action::Raise(amount) => format!("raises to {amount}bb"),
        Action::Shove(amount) => format!("shoves {amount}bb"),
        Action::Blind(amount) => format!("posts {amount}bb"),
        Action::Draw(hand) => format!("draws {}", cards_text(&Vec::<Card>::from(hand))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ScriptedBackend {
        actions: Vec<Action>,
    }

    impl ScriptedBackend {
        fn new(actions: Vec<Action>) -> Self {
            Self { actions }
        }
    }

    impl BotBackend for ScriptedBackend {
        fn strategy_name(&self) -> &str {
            "scripted"
        }

        fn action_distribution(&self, recall: &dyn Recall, _seat: usize) -> Vec<(Action, f64)> {
            let fallback = recall
                .head()
                .legal()
                .first()
                .copied()
                .unwrap_or(Action::Check);
            let action = self.actions.first().copied().unwrap_or(fallback);
            vec![(sanitize_action(&recall.head(), action), 1.0)]
        }

        fn select_action(&self, recall: &dyn Recall, seat: usize) -> Action {
            self.action_distribution(recall, seat)[0].0
        }
    }

    fn passive_action(game: &Game) -> Action {
        if game.may_check() {
            Action::Check
        } else if game.may_call() {
            game.calls()
        } else if game.may_shove() {
            game.shove()
        } else {
            Action::Fold
        }
    }

    #[test]
    fn hand_completes_fold() {
        let backend = Arc::new(ScriptedBackend::new(vec![Action::Check]));
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);

        table
            .apply_hero_action(Action::Fold)
            .expect("fold should be legal");

        assert!(table.is_terminal());
        assert_eq!(table.practice_chips(), DEFAULT_PRACTICE_CHIPS - 1);
        assert_eq!(table.last_result().expect("result").hero_delta, -1);
    }

    #[test]
    fn hand_completes_showdown() {
        let backend = Arc::new(ScriptedBackend::new(vec![Action::Check]));
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);

        table
            .apply_hero_action(Action::Call(1))
            .expect("hero call should be legal");

        while !table.is_terminal() {
            table
                .apply_hero_action(passive_action(table.game()))
                .expect("hero passive line should stay legal");
        }

        assert!(table.last_result().expect("terminal result").showdown);
        assert!(
            table
                .history()
                .iter()
                .any(|action| matches!(action, Action::Draw(_)))
        );
    }

    #[test]
    fn deal_command_sets_cards() {
        let backend = Arc::new(ScriptedBackend::new(vec![Action::Check]));
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);

        table.handle_command("/deal A♠ K♥").expect("deal command");
        table.apply_hero_action(Action::Fold).expect("finish hand");
        table.start_next_hand().expect("next hand");

        assert_eq!(
            table.game().seats()[table.hero_seat()].cards(),
            parse_hole("A♠ K♥").expect("parse target hole")
        );
    }

    #[test]
    fn bot_backend_fallback() {
        let table = TrainingTable::with_fallback_reason("blueprint not found");

        assert_eq!(
            table.strategy_status(),
            "bot strategy: heuristic · blueprint not found"
        );
        assert_eq!(table.bot_backend.strategy_name(), "heuristic");
    }

    #[test]
    fn practice_chips_update() {
        let backend = Arc::new(ScriptedBackend::new(vec![Action::Check]));
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);

        let start = table.practice_chips();
        table.apply_hero_action(Action::Fold).expect("finish hand");

        assert_eq!(table.practice_chips(), start - 1);
    }

    #[test]
    fn alternating_button() {
        let backend = Arc::new(ScriptedBackend::new(vec![Action::Check]));
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);

        assert_eq!(table.hero_position_label(), "BTN");
        table
            .apply_hero_action(Action::Fold)
            .expect("finish first hand");
        table.start_next_hand().expect("next hand");

        assert_eq!(table.hero_position_label(), "BB");
    }
}
