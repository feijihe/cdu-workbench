mod app;
mod utils;
// mod models::modbus_client::{ModbusClient};

fn main() {
    // let ctx = models::modbus_client::ModbusClient::new();
    // models::modbus_client::main();

    app::server::main().unwrap();
    // let client = ModbusClient::new("192.168.1.150", 5000);
    let file_stroe = utils::file_stroe::FileStore::new("config.json", None);
}
