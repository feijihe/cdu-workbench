mod app;
mod utils;
// use serde_json::Value;
// use tokio_modbus::prelude::SyncReader;
// mod models::modbus_client::{ModbusClient};
mod models;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let ctx = models::modbus_client::ModbusClient::new();
    // models::modbus_client::main();

    // app::server::main().unwrap();
    // let client = ModbusClient::new("192.168.1.150", 5000);
    let _file_store =
        utils::file_store::FileStore::new("configs/sensors.yaml", None::<&str>);

    if let Err(e) = _file_store {
        println!("Error: {:?}", e);
        return Ok(());
    }

    let file_store = _file_store.unwrap();
    // let single = file_store.get::<Value>("single").unwrap();
    // let cv = single.as_object().unwrap()["Cv"].clone();
    println!("{:?}", file_store.config);



    // 注释掉同步的 Modbus 客户端代码，避免运行时冲突
    // let mut tcp_client: models::modbus_client::ModbusTcpClient = models::modbus_client::ModbusTcpClient::new("192.168.1.150", 5000);
    // let buff = tcp_client.get_context().read_holding_registers(100, 10).unwrap();
    // println!("====={:?}", buff);

    let time = utils::datetime::get_current_time();
    println!("time: {}", time);
    let server = app::server::Server::new();
    server.run("0.0.0.0", "8080").await
}
