use server::Server;

pub mod ai;
pub mod cards;
pub mod game;
pub mod server;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    
    let server = Server::new();
    server.add_test_game(0);
    server.serve(([127, 0, 0, 1], 3030)).await;
}
