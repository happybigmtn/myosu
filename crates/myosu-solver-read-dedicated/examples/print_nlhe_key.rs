use myosu_games_poker::NlheInfoKey;
use rbp_gameplay::{Abstraction, Edge, Odds, Path};
use rbp_nlhe::NlheInfo;

fn main() {
    let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
        .into_iter()
        .collect::<Path>();
    let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
        .into_iter()
        .collect::<Path>();
    let bucket = Abstraction::from(42_i16);
    let info = NlheInfo::from((subgame, bucket, choices));
    let key = NlheInfoKey::from(&info);
    println!("{}", serde_json::to_string_pretty(&key).unwrap());
}
