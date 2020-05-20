use serde::Deserializer;
use std::collections::{hash_map::Entry, HashMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::{
    fmt::Display,
    str::FromStr,
    time::{Duration, Instant},
};

use futures::{stream, StreamExt};
use serde::{de, Deserialize};
use tokio::{sync::broadcast, time::interval};
use warp::{path, path::param, query, reject, sse, Filter, Rejection, Reply};

use crate::ai::AIPlayer;
use crate::game::{GameState, PlayerAction, PlayerActionResult};

#[derive(Debug)]
struct Game {
    creation_time: Instant,
    notify_change: broadcast::Sender<GameEvent>,
    inner: Mutex<GameInner>,
}

#[derive(Debug)]
struct GameInner {
    state: GameState,
    players: Vec<Player>,
    is_started: bool,
}

impl GameInner {
    pub fn get_player(&self, auth: &str) -> Option<usize> {
        let (player, _) = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if let Player::RealPlayer(p) = p {
                    Some((i, p))
                } else {
                    None
                }
            })
            .find(|(_, p)| p == &auth)?;
        Some(player)
    }
}

impl Game {
    pub fn new(ai_player_count: usize) -> Arc<Game> {
        let (sender, _) = broadcast::channel(16);
        assert!(ai_player_count <= 4);
        Arc::new(Game {
            creation_time: Instant::now(),
            notify_change: sender,
            inner: Mutex::new(GameInner {
                state: GameState::initial(),
                players: (0..ai_player_count)
                    .map(|x| Player::AI(AIPlayer::new(x)))
                    .collect(),
                is_started: false,
            }),
        })
    }

    pub fn join_player(self: &Arc<Self>) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        if inner.players.len() >= 4 {
            None?
        }
        let id = inner.players.len();
        let credentials = format!("{}:{:016x}", id, rand::random::<u64>());
        inner
            .players
            .push(Player::RealPlayer(base64::encode(&credentials)));
        drop(inner);
        self.check_start_game();
        Some(credentials)
    }

    fn broadcast(self: &Arc<Self>, event: GameEvent) {
        let _ = self.notify_change.send(event);
    }

    // returns true of the game is won.
    pub fn perform_player_action(self: &Arc<Self>, player: usize, action: PlayerAction) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let result = inner
            .state
            .perform_player_action(player, action.clone())
            .unwrap();

        let state = inner.state.clone();
        drop(inner);

        self.broadcast(GameEvent::GameStateChanged(state));

        match result {
            PlayerActionResult::Nominal => false,
            PlayerActionResult::NextPlayer(_) => {
                self.check_play_ai();
                false
            }
            PlayerActionResult::GameWon(winner) => {
                self.broadcast(GameEvent::GameWon(winner));
                true
            }
        }
    }

    pub fn check_start_game(self: &Arc<Self>) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.is_started && inner.players.len() == 4 {
            let state = inner.state.clone();
            inner.is_started = true;
            drop(inner);

            self.broadcast(GameEvent::GameStateChanged(state));
            self.check_play_ai();
        }
    }

    pub fn check_play_ai(self: &Arc<Self>) {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_started {
            let current = inner.state.round_state.player as usize;
            let state = inner.state.clone();
            let players = &mut inner.players;
            if let Player::AI(ai) = &mut players[current] {
                let moves = ai.play_turn(state);
                let self2 = self.clone();
                drop(inner);
                tokio::spawn(async move {
                    let mut interval = interval(Duration::from_secs(1));
                    interval.tick().await;
                    for m in moves {
                        interval.tick().await;
                        if self2.perform_player_action(current, m) {
                            return;
                        }
                    }
                });
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Player {
    AI(AIPlayer),
    RealPlayer(String),
}

#[derive(Clone, Debug)]
enum GameEvent {
    GameStateChanged(GameState),
    GameWon(usize),
}

impl GameEvent {
    fn to_string(&self, player: usize) -> String {
        match self {
            GameEvent::GameStateChanged(state) => {
                if player == state.round_state.player {
                    format!(
                        "state:{}\nhand:{}",
                        state.to_string(),
                        state.hand_to_string()
                    )
                } else {
                    format!("state:{}", state.to_string())
                }
            }
            GameEvent::GameWon(winner) => format!("gmwon:{}", winner),
        }
    }
}

#[derive(Clone, Default)]
pub struct Server {
    games: Arc<RwLock<HashMap<u64, Arc<Game>>>>,
}

#[derive(Debug)]
enum ServerError {
    PathError,
    InternalError,
    GameNotFound,
    InvalidAuth,
}
impl Display for ServerError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        <Self as std::fmt::Debug>::fmt(self, fmt)
    }
}
impl reject::Reject for ServerError {}
impl std::error::Error for ServerError {}
impl From<ServerError> for Rejection {
    fn from(err: ServerError) -> Rejection {
        reject::custom(err)
    }
}

impl Server {
    pub fn new() -> Server {
        Default::default()
    }

    pub fn add_test_game(&self, id: u64) {
        let game = Game::new(4);
        game.check_start_game();
        self.games.write().unwrap().entry(id).or_insert(game);
    }

    fn get_game_filter(&self) -> impl Filter<Extract = (Arc<Game>,), Error = Rejection> + Clone {
        async fn get_game(this: Server, game_id: String) -> Result<Arc<Game>, Rejection> {
            let id = u64::from_str_radix(&game_id, 16).map_err(|_| ServerError::PathError)?;
            let games = this.games.read().unwrap();
            let game = games.get(&id).ok_or(ServerError::GameNotFound)?;
            Ok(game.clone())
        }
        let this = self.clone();
        param().and_then(move |game_id| get_game(this.clone(), game_id))
    }

    fn auth_filter(&self) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
        async fn parse_auth(auth: String) -> Result<String, Rejection> {
            let mut iter = auth.split_ascii_whitespace();
            let auth_type = iter.next().ok_or(ServerError::InvalidAuth)?;
            let auth_str = iter.next().ok_or(ServerError::InvalidAuth)?;
            if auth_type != "Basic" || iter.next().is_some() {
                Err(ServerError::InvalidAuth)?
            }
            Ok(auth_str.to_owned())
        }
        warp::header("Authorization").and_then(|auth: String| parse_auth(auth))
    }

    fn map_game_event_stream(game: Arc<Game>, auth: &str) -> Result<impl Reply, Rejection> {
        let inner = game.inner.lock().unwrap();

        let player = inner.get_player(auth).ok_or(ServerError::InvalidAuth)?;

        let state = inner.state.clone();
        drop(inner);

        let event_stream = game.notify_change.subscribe();
        let state_stream = stream::once(async move { Ok(GameEvent::GameStateChanged(state)) });
        let both = stream::select(state_stream, event_stream);

        Ok(sse::reply(both.map(move |event| match event {
            Ok(event) => Ok(sse::data(format!("{}", event.to_string(player)))),
            Err(_) => Err(ServerError::InternalError),
        })))
    }

    fn create_game(&self, ai_player_count: u8) -> u64 {
        let game = Game::new(ai_player_count as usize);
        game.check_start_game();
        loop {
            let id = rand::random();
            match self.games.write().unwrap().entry(id) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(v) => v.insert(game),
            };
            break id;
        }
    }

    pub async fn serve(&self, addr: impl Into<SocketAddr> + 'static) {
        let log = warp::log("web_api");

        // GET server.com/api/v0/game/stream/123abc/ (with basic Auth)
        let stream = path("stream")
            .and(self.get_game_filter())
            .and(path::end())
            .and(warp::get())
            .and(self.auth_filter())
            .and_then(|x, auth: String| async move { Server::map_game_event_stream(x, &auth) });
        // POST server.com/api/v0/game/join/123abc/
        let join = path("join")
            .and(self.get_game_filter())
            .and(path::end())
            .and(warp::post())
            .map(|game: Arc<Game>| game.join_player().unwrap_or("Error".to_string()));

        // POST server.com/api/v0/game/action/123abc/?action=dscd (with basic Auth)
        let action = path("action")
            .and(self.get_game_filter())
            .and(path::end())
            .and(warp::post())
            .and(self.auth_filter())
            .and(query())
            .and_then(
                |game: Arc<Game>, auth: String, query: ActionQuery| async move {
                    println!("{},{:?}", auth, &query);
                    let inner = game.inner.lock().unwrap();
                    let player = inner.get_player(&auth).ok_or(ServerError::InvalidAuth)?;
                    drop(inner);
                    game.perform_player_action(player, query.action);

                    let result: Result<&'static str, Rejection> = Ok("success");
                    result
                },
            );

        let self2 = self.clone();
        // POST server.com/api/v0/game/create/?ai_players=3
        let create = path!("create")
            .and(warp::post())
            .and(query())
            .map(move |query: CreateQuery| format!("{:016x}", self2.create_game(query.ai_players)));

        let game = path("game").and(stream.or(join).or(action));

        let api = path!("api" / "v0" / ..).and(game.or(create)).with(log);

        warp::serve(api).run(addr).await;
    }
}

#[derive(Deserialize)]
struct CreateQuery {
    ai_players: u8,
}

#[derive(Debug, Deserialize)]
struct ActionQuery {
    #[serde(deserialize_with = "str_to_player_action")]
    action: PlayerAction,
}

fn str_to_player_action<'de, D>(deserializer: D) -> Result<PlayerAction, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    println!("{:?}", &s);
    FromStr::from_str(&s).map_err(|_| de::Error::custom("Error while deserializing PlayerAction"))
}
