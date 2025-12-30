use tokio_modbus::{
    prelude::{sync, SyncReader},
    ExceptionCode, Slave,
};

// let mut client = SerialClient::new("/dev/ttyUSB0", 1);

pub fn main() {
    let socket_addr = "192.168.1.150:5000".parse().unwrap();
    let mut ctx: sync::Context = sync::tcp::connect_slave(socket_addr, Slave(1)).unwrap();
    // let buff = ctx.read_holding_registers(0, 10);
    // match buff {
    //     Ok(res) => println!("====={:?}", res),
    //     Err(e) => println!("====={:?}", e),
    // }
    // println!("====={:?}", buff);
}

pub struct ModbusClient {
    client: sync::Context,
}

impl ModbusClient {
    pub fn new(ip: &str, port: u16) -> Self {
        let socket_addr = format!("{}:{}", ip, port).parse().unwrap();
        let ctx: sync::Context = sync::tcp::connect_slave(socket_addr, Slave(1)).unwrap();
        Self { client: ctx }
    }
    pub fn connected(&self) -> bool {
        true
    }
}
