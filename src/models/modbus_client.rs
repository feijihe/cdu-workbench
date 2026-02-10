use tokio_modbus::{
    prelude::{sync, SyncReader},
    ExceptionCode, Slave,
};
// use serialport::SerialPortBuilder;

pub struct ModbusTcpClient {
    pub client: sync::Context,
}

pub struct ModbusSerialClient {
    client: sync::Context,
}

impl ModbusTcpClient {
    #[allow(dead_code)]
    pub fn new(ip: &str, port: u16) -> Self {
        let socket_addr = format!("{}:{}", ip, port).parse().unwrap();
        let ctx: sync::Context = sync::tcp::connect_slave(socket_addr, Slave(1)).unwrap();
        Self { client: ctx }
    }
    #[allow(dead_code)]
    pub fn get_context(&mut self) -> &mut sync::Context {
        &mut self.client
    }
}

impl ModbusSerialClient {
    #[allow(dead_code)]
    pub fn new(port: &str, baud_rate: u32) -> Self {
        let builder = serialport::new(port, baud_rate);
        let ctx: sync::Context = sync::rtu::connect_slave(&builder, Slave(1)).unwrap();
        Self { client: ctx }
    }
    #[allow(dead_code)]
    pub fn get_context(&mut self) -> &mut sync::Context {
        &mut self.client
    }
}
