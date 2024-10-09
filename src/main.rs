use server::Server;

mod vm;
mod program;
mod repositories;
mod server;

fn main() {
    let server = Server::new();
    server.listen();
}
