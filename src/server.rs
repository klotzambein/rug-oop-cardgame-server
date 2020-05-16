use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use serde::Deserialize;
use warp::{path, path::param, query, reject, Filter, Rejection};

#[derive(Debug)]
struct Game {
    creation_time: Instant,
}

impl Game {
    pub fn new() -> Arc<Game> {
        Arc::new(Game {
            creation_time: Instant::now(),
        })
    }
}

#[derive(Clone, Default)]
pub struct Server {
    games: Arc<RwLock<HashMap<u64, Arc<Game>>>>,
}

#[derive(Debug)]
enum ServerError {
    PathError,
    GameNotFound,
}
impl reject::Reject for ServerError {}
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
        self.games.write().unwrap().entry(id).or_insert(Game::new());
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

    pub async fn serve(&self, addr: impl Into<SocketAddr> + 'static) {
        let game = path!("game" / ..)
            .and(self.get_game_filter())
            .map(|game| format!("{:#?}", game));

        let create = path!("create")
            .and(warp::post())
            .and(query())
            .map(|query: CreateQuery| format!("{}", query.ai_players));

        let api = path!("api" / "v0" / ..).and(game.or(create));

        warp::serve(api).run(addr).await;
    }
}

#[derive(Deserialize)]
struct CreateQuery {
    ai_players: u8,
}
