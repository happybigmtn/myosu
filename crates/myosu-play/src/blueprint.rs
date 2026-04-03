use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use myosu_games::{CfrGame, CfrInfo};
use myosu_games_kuhn::{KuhnRenderer, KuhnSnapshot};
use myosu_games_liars_dice::{
    LiarsDiceClaim, LiarsDiceGame, LiarsDiceRenderer, LiarsDiceSnapshot, LiarsDiceSolver,
    recommended_edge as recommended_liars_dice_edge,
};
use myosu_games_poker::{CodexpokerBlueprint, NlheRenderer, PokerSolver, load_encoder_dir};
use myosu_tui::GameRenderer;

use crate::cli::{AdviceArgs, GameSelection};

const LIARS_DICE_SOLVER_TREES: usize = 1 << 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdviceMode {
    Standard,
    SmokeTest,
}

pub(crate) enum AdviceSurface {
    Poker { renderer: Arc<NlheRenderer> },
    Kuhn { renderer: Arc<KuhnRenderer> },
    LiarsDice { renderer: Arc<LiarsDiceRenderer> },
}

impl AdviceSurface {
    pub(crate) fn renderer(&self) -> &dyn GameRenderer {
        match self {
            Self::Poker { renderer } => renderer.as_ref(),
            Self::Kuhn { renderer } => renderer.as_ref(),
            Self::LiarsDice { renderer } => renderer.as_ref(),
        }
    }

    pub(crate) fn poker_renderer(&self) -> Option<&NlheRenderer> {
        match self {
            Self::Poker { renderer } => Some(renderer.as_ref()),
            Self::Kuhn { .. } => None,
            Self::LiarsDice { .. } => None,
        }
    }

    pub(crate) fn poker_renderer_arc(&self) -> Option<Arc<NlheRenderer>> {
        match self {
            Self::Poker { renderer } => Some(renderer.clone()),
            Self::Kuhn { .. } => None,
            Self::LiarsDice { .. } => None,
        }
    }
}

pub(crate) struct AdviceSelection {
    pub(crate) game: GameSelection,
    pub(crate) surface: AdviceSurface,
    pub(crate) source: &'static str,
    pub(crate) selection: &'static str,
    pub(crate) origin: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdviceStartupState {
    Success,
    Empty,
    Partial,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AdviceAssets {
    checkpoint: PathBuf,
    encoder_dir: PathBuf,
    origin: BlueprintRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BlueprintRoot {
    kind: &'static str,
    path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BlueprintRootDiagnostic {
    origin: BlueprintRoot,
    detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AutoBlueprintResolution {
    Found(AdviceAssets),
    Incomplete(BlueprintRootDiagnostic),
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BlueprintRootResolution {
    Found(AdviceAssets),
    Incomplete(BlueprintRootDiagnostic),
    Missing,
}

impl AdviceSelection {
    pub(crate) fn startup_state(&self) -> AdviceStartupState {
        match (self.source, self.reason) {
            ("demo", _) => AdviceStartupState::Success,
            ("artifact", _) => AdviceStartupState::Success,
            ("generated", "missing") => AdviceStartupState::Empty,
            ("generated", _) => AdviceStartupState::Partial,
            _ => AdviceStartupState::Partial,
        }
    }

    pub(crate) fn startup_messages(&self) -> Vec<String> {
        match self.startup_state() {
            AdviceStartupState::Success => match self.source {
                "demo" => vec![format!("Loaded {}.", self.detail)],
                _ => vec![format!("Loaded local advice artifacts from {}.", self.origin)],
            },
            AdviceStartupState::Empty => vec![
                "No local blueprint artifacts found. You can still explore the demo hand with generated advice.".to_string(),
                "To use saved solver advice, pass --checkpoint FILE --encoder-dir DIR.".to_string(),
                "Or set MYOSU_BLUEPRINT_DIR to a folder containing latest.bin and manifest.json.".to_string(),
            ],
            AdviceStartupState::Partial => vec![
                "Artifact advice is unavailable, so the table is using generated demo advice."
                    .to_string(),
                format!("Artifact detail: {}.", self.detail),
            ],
        }
    }

    pub(crate) fn info_line(&self) -> String {
        format!(
            "INFO protocol=text_demo protocol_version=1 advice_source={} selection={} origin={} reason={} detail={:?}",
            self.source, self.selection, self.origin, self.reason, self.detail
        )
    }

    pub(crate) fn summary_line(&self) -> String {
        format!(
            "Advice source: {} [selection={} origin={} reason={}] ({}).",
            self.source, self.selection, self.origin, self.reason, self.detail
        )
    }
}

pub(crate) fn demo_renderer(game: GameSelection, args: AdviceArgs) -> io::Result<AdviceSelection> {
    demo_renderer_with_mode(game, args, AdviceMode::Standard)
}

pub(crate) fn smoke_demo_renderer(
    game: GameSelection,
    args: AdviceArgs,
) -> io::Result<AdviceSelection> {
    demo_renderer_with_mode(game, args, AdviceMode::SmokeTest)
}

fn demo_renderer_with_mode(
    game: GameSelection,
    args: AdviceArgs,
    mode: AdviceMode,
) -> io::Result<AdviceSelection> {
    match game {
        GameSelection::Poker => poker_demo_renderer(args, mode),
        GameSelection::Kuhn => kuhn_demo_renderer(args),
        GameSelection::LiarsDice => liars_dice_demo_renderer(args),
    }
}

fn poker_demo_renderer(args: AdviceArgs, mode: AdviceMode) -> io::Result<AdviceSelection> {
    match (args.checkpoint, args.encoder_dir) {
        (Some(checkpoint), Some(encoder_dir)) => {
            let renderer = load_blueprint_renderer(&checkpoint, &encoder_dir)?;
            Ok(AdviceSelection {
                game: GameSelection::Poker,
                surface: AdviceSurface::Poker {
                    renderer: renderer.clone(),
                },
                source: "artifact",
                selection: "explicit",
                origin: "explicit",
                reason: "explicit",
                detail: format!(
                    "explicit checkpoint={} encoder_dir={}",
                    checkpoint.display(),
                    encoder_dir.display()
                ),
            })
        }
        (None, None) => {
            if mode == AdviceMode::SmokeTest {
                return Ok(AdviceSelection {
                    game: GameSelection::Poker,
                    surface: AdviceSurface::Poker {
                        renderer: Arc::new(NlheRenderer::demo()),
                    },
                    source: "demo",
                    selection: "smoke",
                    origin: "builtin",
                    reason: "smoke_test",
                    detail: "built-in poker demo surface for smoke proof".to_string(),
                });
            }

            if let Some(directory) = auto_codexpoker_blueprint_dir() {
                match CodexpokerBlueprint::load(&directory) {
                    Ok(blueprint) => {
                        return Ok(AdviceSelection {
                            game: GameSelection::Poker,
                            surface: AdviceSurface::Poker {
                                renderer: Arc::new(NlheRenderer::demo_with_codexpoker_blueprint(
                                    blueprint,
                                )),
                            },
                            source: "artifact",
                            selection: "auto",
                            origin: "codexpoker_home",
                            reason: "auto_loaded",
                            detail: format!(
                                "auto codexpoker_home artifact_dir={}",
                                directory.display()
                            ),
                        });
                    }
                    Err(error) => {
                        return Ok(AdviceSelection {
                            game: GameSelection::Poker,
                            surface: AdviceSurface::Poker {
                                renderer: Arc::new(NlheRenderer::demo()),
                            },
                            source: "generated",
                            selection: "fallback",
                            origin: "codexpoker_home",
                            reason: "load_failed",
                            detail: format!("auto codexpoker_home blueprint load failed: {error}"),
                        });
                    }
                }
            }

            match auto_blueprint_assets() {
                AutoBlueprintResolution::Found(assets) => {
                    match load_blueprint_renderer(&assets.checkpoint, &assets.encoder_dir) {
                        Ok(renderer) => Ok(AdviceSelection {
                            game: GameSelection::Poker,
                            surface: AdviceSurface::Poker {
                                renderer: renderer.clone(),
                            },
                            source: "artifact",
                            selection: "auto",
                            origin: assets.origin.kind,
                            reason: "auto_loaded",
                            detail: format!(
                                "auto {} checkpoint={} encoder_dir={}",
                                assets.origin.kind,
                                assets.checkpoint.display(),
                                assets.encoder_dir.display()
                            ),
                        }),
                        Err(error) => Ok(AdviceSelection {
                            game: GameSelection::Poker,
                            surface: AdviceSurface::Poker {
                                renderer: Arc::new(NlheRenderer::demo()),
                            },
                            source: "generated",
                            selection: "fallback",
                            origin: assets.origin.kind,
                            reason: "load_failed",
                            detail: format!(
                                "auto {} blueprint load failed: {error}",
                                assets.origin.kind
                            ),
                        }),
                    }
                }
                AutoBlueprintResolution::Incomplete(diagnostic) => Ok(AdviceSelection {
                    game: GameSelection::Poker,
                    surface: AdviceSurface::Poker {
                        renderer: Arc::new(NlheRenderer::demo()),
                    },
                    source: "generated",
                    selection: "fallback",
                    origin: diagnostic.origin.kind,
                    reason: "incomplete",
                    detail: diagnostic.detail,
                }),
                AutoBlueprintResolution::Missing => Ok(AdviceSelection {
                    game: GameSelection::Poker,
                    surface: AdviceSurface::Poker {
                        renderer: Arc::new(NlheRenderer::demo()),
                    },
                    source: "generated",
                    selection: "fallback",
                    origin: "none",
                    reason: "missing",
                    detail: "no local blueprint artifacts found".to_string(),
                }),
            }
        }
        (Some(_), None) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--checkpoint requires --encoder-dir",
        )),
        (None, Some(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--encoder-dir requires --checkpoint",
        )),
    }
}

fn liars_dice_demo_renderer(args: AdviceArgs) -> io::Result<AdviceSelection> {
    match (args.checkpoint, args.encoder_dir) {
        (Some(checkpoint), None) => {
            let renderer = load_liars_dice_renderer(&checkpoint)?;
            Ok(AdviceSelection {
                game: GameSelection::LiarsDice,
                surface: AdviceSurface::LiarsDice {
                    renderer: renderer.clone(),
                },
                source: "artifact",
                selection: "explicit",
                origin: "explicit",
                reason: "explicit",
                detail: format!("explicit checkpoint={}", checkpoint.display()),
            })
        }
        (None, None) => Ok(AdviceSelection {
            game: GameSelection::LiarsDice,
            surface: AdviceSurface::LiarsDice {
                renderer: Arc::new(LiarsDiceRenderer::new(Some(LiarsDiceSnapshot::demo()))),
            },
            source: "demo",
            selection: "builtin",
            origin: "builtin",
            reason: "builtin",
            detail: "built-in liar's dice demo surface".to_string(),
        }),
        (Some(_), Some(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--game liars-dice does not use --encoder-dir",
        )),
        (None, Some(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--game liars-dice does not use --encoder-dir",
        )),
    }
}

fn kuhn_demo_renderer(args: AdviceArgs) -> io::Result<AdviceSelection> {
    match (args.checkpoint, args.encoder_dir) {
        (None, None) => Ok(AdviceSelection {
            game: GameSelection::Kuhn,
            surface: AdviceSurface::Kuhn {
                renderer: Arc::new(KuhnRenderer::new(Some(KuhnSnapshot::demo()))),
            },
            source: "demo",
            selection: "builtin",
            origin: "builtin",
            reason: "builtin",
            detail: "built-in kuhn poker demo surface".to_string(),
        }),
        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--game kuhn does not use --checkpoint/--encoder-dir",
        )),
    }
}

fn auto_codexpoker_blueprint_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    auto_codexpoker_blueprint_dir_from(home)
}

fn auto_codexpoker_blueprint_dir_from(home: Option<PathBuf>) -> Option<PathBuf> {
    let home = home?;
    let directory = home.join(".codexpoker").join("blueprint");
    directory
        .join("blueprint.manifest.json")
        .is_file()
        .then_some(directory)
}

fn load_blueprint_renderer(
    checkpoint: &PathBuf,
    encoder_dir: &PathBuf,
) -> io::Result<Arc<NlheRenderer>> {
    let encoder = load_encoder_dir(encoder_dir).map_err(|error| {
        io::Error::other(format!(
            "failed to load encoder dir `{}`: {error}",
            encoder_dir.display()
        ))
    })?;
    let solver = PokerSolver::load(checkpoint, encoder).map_err(|error| {
        io::Error::other(format!(
            "failed to load checkpoint `{}`: {error}",
            checkpoint.display()
        ))
    })?;
    let blueprint = solver
        .blueprint()
        .map_err(|error| io::Error::other(format!("failed to snapshot blueprint: {error}")))?;
    Ok(Arc::new(NlheRenderer::demo_with_blueprint(blueprint)))
}

fn load_liars_dice_renderer(checkpoint: &PathBuf) -> io::Result<Arc<LiarsDiceRenderer>> {
    let solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(checkpoint).map_err(|error| {
        io::Error::other(format!(
            "failed to load liar's dice checkpoint `{}`: {error}",
            checkpoint.display()
        ))
    })?;
    let snapshot = liars_dice_snapshot_from_solver(&solver)?;
    Ok(Arc::new(LiarsDiceRenderer::new(Some(snapshot))))
}

fn liars_dice_snapshot_from_solver(
    solver: &LiarsDiceSolver<LIARS_DICE_SOLVER_TREES>,
) -> io::Result<LiarsDiceSnapshot> {
    let opening =
        LiarsDiceGame::root().apply(myosu_games_liars_dice::LiarsDiceEdge::Roll { p1: 4, p2: 2 });
    let info = opening.info().ok_or_else(|| {
        io::Error::other("liar's dice opening state did not expose player information")
    })?;
    let response = solver.query(info);
    let recommendation = recommended_liars_dice_edge(&response).map(liars_dice_action_label);
    let last_claim = info.public().last_claim();
    let legal_actions = liars_dice_legal_actions(last_claim);
    let context = match recommendation {
        Some(recommendation) => format!("ROUND 1 | ADVICE {}", recommendation.to_ascii_uppercase()),
        None => "ROUND 1".to_string(),
    };

    Ok(LiarsDiceSnapshot {
        your_die: info.secret().0,
        last_claim,
        legal_actions,
        context,
    })
}

fn liars_dice_legal_actions(last_claim: Option<LiarsDiceClaim>) -> Vec<String> {
    let mut actions = Vec::new();
    for count in 1..=2 {
        for face in 1..=6 {
            let claim = LiarsDiceClaim::new(count, face).expect("claim should be valid");
            if last_claim.is_some_and(|current| claim <= current) {
                continue;
            }
            actions.push(format!("bid {count}x{face}"));
        }
    }
    if last_claim.is_some() {
        actions.push("liar".to_string());
    }
    actions
}

fn liars_dice_action_label(edge: myosu_games_liars_dice::LiarsDiceEdge) -> String {
    match edge {
        myosu_games_liars_dice::LiarsDiceEdge::Bid(claim) => {
            format!("bid {}x{}", claim.count, claim.face)
        }
        myosu_games_liars_dice::LiarsDiceEdge::Challenge => "liar".to_string(),
        myosu_games_liars_dice::LiarsDiceEdge::Roll { .. } => "roll".to_string(),
    }
}

fn auto_blueprint_assets() -> AutoBlueprintResolution {
    let cwd = std::env::current_dir().ok();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_dir = std::env::var_os("MYOSU_DATA_DIR").map(PathBuf::from);
    let env_root = std::env::var_os("MYOSU_BLUEPRINT_DIR").map(PathBuf::from);

    auto_blueprint_assets_from(cwd, home, data_dir, env_root)
}

fn auto_blueprint_assets_from(
    cwd: Option<PathBuf>,
    home: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    env_root: Option<PathBuf>,
) -> AutoBlueprintResolution {
    let mut roots = Vec::new();
    if let Some(root) = env_root {
        roots.push(BlueprintRoot {
            kind: "env",
            path: root,
        });
    }
    if let Some(root) = data_dir {
        roots.push(BlueprintRoot {
            kind: "data",
            path: root.join(".myosu").join("blueprints"),
        });
    }
    if let Some(root) = cwd {
        roots.push(BlueprintRoot {
            kind: "repo",
            path: root.join("artifacts"),
        });
    }
    if let Some(root) = home {
        roots.push(BlueprintRoot {
            kind: "home",
            path: root.join(".myosu").join("blueprints"),
        });
    }

    let mut first_incomplete = None;
    for root in roots {
        match resolve_blueprint_assets(&root) {
            BlueprintRootResolution::Found(assets) => {
                return AutoBlueprintResolution::Found(assets);
            }
            BlueprintRootResolution::Incomplete(diagnostic) => {
                if first_incomplete.is_none() {
                    first_incomplete = Some(diagnostic);
                }
            }
            BlueprintRootResolution::Missing => {}
        }
    }

    match first_incomplete {
        Some(detail) => AutoBlueprintResolution::Incomplete(detail),
        None => AutoBlueprintResolution::Missing,
    }
}

fn resolve_blueprint_assets(root: &BlueprintRoot) -> BlueprintRootResolution {
    if !root.path.exists() {
        return BlueprintRootResolution::Missing;
    }

    let split = AdviceAssets {
        checkpoint: root.path.join("checkpoints").join("latest.bin"),
        encoder_dir: root.path.join("abstractions"),
        origin: root.clone(),
    };
    if split.checkpoint.is_file() && split.encoder_dir.join("manifest.json").is_file() {
        return BlueprintRootResolution::Found(split);
    }

    let direct = AdviceAssets {
        checkpoint: root.path.join("latest.bin"),
        encoder_dir: root.path.clone(),
        origin: root.clone(),
    };
    if direct.checkpoint.is_file() && direct.encoder_dir.join("manifest.json").is_file() {
        return BlueprintRootResolution::Found(direct);
    }

    BlueprintRootResolution::Incomplete(BlueprintRootDiagnostic {
        origin: root.clone(),
        detail: format!(
            "incomplete blueprint artifacts under {} (expected checkpoints/latest.bin + abstractions/manifest.json or latest.bin + manifest.json)",
            root.path.display()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_renderer_requires_complete_advice_pairing() {
        let missing_encoder = demo_renderer(
            GameSelection::Poker,
            AdviceArgs {
                checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
                encoder_dir: None,
            },
        );
        let missing_encoder = match missing_encoder {
            Ok(_) => panic!("checkpoint without encoder dir should fail"),
            Err(error) => error,
        };
        assert_eq!(missing_encoder.kind(), io::ErrorKind::InvalidInput);

        let missing_checkpoint = demo_renderer(
            GameSelection::Poker,
            AdviceArgs {
                checkpoint: None,
                encoder_dir: Some(PathBuf::from("/tmp/encoder-dir")),
            },
        );
        let missing_checkpoint = match missing_checkpoint {
            Ok(_) => panic!("encoder dir without checkpoint should fail"),
            Err(error) => error,
        };
        assert_eq!(missing_checkpoint.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn auto_codexpoker_blueprint_dir_uses_home_layout() {
        let root = std::env::temp_dir().join("myosu-play-codexpoker-home");
        let directory = root.join(".codexpoker").join("blueprint");
        std::fs::create_dir_all(&directory).expect("test directory should write");
        std::fs::write(directory.join("blueprint.manifest.json"), "{}")
            .expect("manifest should write");

        let discovered =
            auto_codexpoker_blueprint_dir_from(Some(root.clone())).expect("dir should resolve");
        assert_eq!(discovered, directory);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn smoke_demo_renderer_uses_builtin_poker_surface() {
        let selection = smoke_demo_renderer(
            GameSelection::Poker,
            AdviceArgs {
                checkpoint: None,
                encoder_dir: None,
            },
        )
        .expect("smoke poker demo should build");

        assert_eq!(selection.game, GameSelection::Poker);
        assert_eq!(selection.source, "demo");
        assert_eq!(selection.selection, "smoke");
        assert_eq!(selection.origin, "builtin");
        assert_eq!(selection.reason, "smoke_test");
        assert_eq!(selection.startup_state(), AdviceStartupState::Success);
        assert!(
            selection
                .surface
                .renderer()
                .pipe_output()
                .contains("street=PREFLOP")
        );
    }

    #[test]
    fn startup_messages_cover_success_empty_and_partial_states() {
        let success = AdviceSelection {
            game: GameSelection::Poker,
            surface: AdviceSurface::Poker {
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
            surface: AdviceSurface::Poker {
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
            surface: AdviceSurface::Poker {
                renderer: Arc::new(NlheRenderer::demo()),
            },
            source: "generated",
            selection: "fallback",
            origin: "repo",
            reason: "load_failed",
            detail: "checkpoint decode failed".to_string(),
        };

        assert_eq!(success.startup_state(), AdviceStartupState::Success);
        assert_eq!(empty.startup_state(), AdviceStartupState::Empty);
        assert_eq!(partial.startup_state(), AdviceStartupState::Partial);
        assert_eq!(
            empty.startup_messages()[0],
            "No local blueprint artifacts found. You can still explore the demo hand with generated advice."
        );
        assert!(empty.startup_messages()[1].contains("--checkpoint FILE --encoder-dir DIR"));
        assert!(empty.startup_messages()[2].contains("MYOSU_BLUEPRINT_DIR"));
        assert!(partial.startup_messages()[1].contains("checkpoint decode failed"));
    }

    #[test]
    fn liars_dice_demo_renderer_uses_builtin_surface_without_checkpoint() {
        let selection = demo_renderer(
            GameSelection::LiarsDice,
            AdviceArgs {
                checkpoint: None,
                encoder_dir: None,
            },
        )
        .expect("liar's dice demo should build");

        assert_eq!(selection.game, GameSelection::LiarsDice);
        assert_eq!(selection.source, "demo");
        assert_eq!(selection.startup_state(), AdviceStartupState::Success);
        assert!(
            selection
                .surface
                .renderer()
                .pipe_output()
                .contains("game=liars_dice")
        );
    }

    #[test]
    fn liars_dice_demo_renderer_rejects_encoder_dir() {
        let error = match demo_renderer(
            GameSelection::LiarsDice,
            AdviceArgs {
                checkpoint: None,
                encoder_dir: Some(PathBuf::from("/tmp/encoder-dir")),
            },
        ) {
            Ok(_) => panic!("liar's dice should reject encoder dirs"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("does not use --encoder-dir"));
    }

    #[test]
    fn kuhn_demo_renderer_uses_builtin_surface_without_checkpoint() {
        let selection = demo_renderer(
            GameSelection::Kuhn,
            AdviceArgs {
                checkpoint: None,
                encoder_dir: None,
            },
        )
        .expect("kuhn demo should build");

        assert_eq!(selection.game, GameSelection::Kuhn);
        assert_eq!(selection.source, "demo");
        assert_eq!(selection.startup_state(), AdviceStartupState::Success);
        assert_eq!(
            selection.startup_messages(),
            vec!["Loaded built-in kuhn poker demo surface.".to_string()]
        );
        assert!(
            selection
                .surface
                .renderer()
                .pipe_output()
                .contains("game=kuhn_poker")
        );
    }

    #[test]
    fn kuhn_demo_renderer_rejects_checkpoint_flags() {
        let error = match demo_renderer(
            GameSelection::Kuhn,
            AdviceArgs {
                checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
                encoder_dir: None,
            },
        ) {
            Ok(_) => panic!("kuhn should reject checkpoint flags"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(
            error
                .to_string()
                .contains("does not use --checkpoint/--encoder-dir")
        );
    }
}
