use mysql::{self, Row};
use mysql::prelude::Queryable;
use webserver_rust::server::Server;
use webserver_rust::async_server::AsyncSercer;

fn main() {
    let f = |read_buffer: Vec<u8>| -> Vec<u8> {
        let user = "localuser";
        let password = "localuser";
        let ip = "localhost";
        let port = 3306;
        let database = "test";
        let url = format!("mysql://{}:{}@{}:{}/{}", user, password, ip, port, database);
        let pool = match mysql::Pool::new(url.as_str()) {
            Ok(pool) => pool,
            Err(err) => return format!("{:?}", err).into_bytes(),
        };
        let mut connect = match pool.get_conn() {
            Ok(connect) => connect,
            Err(err) => return format!("{:?}", err).into_bytes(),
        };
        let query = match String::from_utf8(read_buffer) {
            Ok(query) => query,
            Err(err) => return format!("{:?}", err).into_bytes(),
        };
        match connect.query::<Row, &str>(query.trim()) {
            Ok(row) => {
                let mut write_buffer = String::new();
                for r in row {
                    write_buffer += format!("{:?}\n", r).as_str();
                }
                write_buffer.into_bytes()
            }
            Err(err) => format!("{:?}", err).into_bytes(),
        }
    };
    let server = AsyncSercer::new(8081, Box::new(f));
    server.start().unwrap();
    /*
    match Server::new(8081, Box::new(f)) {
        Ok(mut server) => {
            server.start();
        }
        Err(e) => panic!("{:?}", e),
    }
    */
}
