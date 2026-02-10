use crate::models::modbus_client::{ModbusClient, ModbusConnectError};

fn main() {
    // 配置参数
    let tcp_ip = "192.168.1.150";
    let tcp_port = 502;
    let rtu_port = "/dev/ttyUSB0";
    let rtu_baud_rate = 9600;
    let slave_id = 1;
    
    println!("正在尝试连接Modbus设备...");
    
    // 创建Modbus客户端
    match ModbusClient::new(tcp_ip, tcp_port, rtu_port, rtu_baud_rate, slave_id) {
        Ok(mut client) => {
            println!("连接成功! 使用的连接类型: {}", client.connection_type());
            
            // 示例：读取保持寄存器
            println!("尝试读取保持寄存器...");
            match client.get_context().read_holding_registers(0, 10) {
                Ok(result) => {
                    println!("读取成功: {:?}", result);
                },
                Err(e) => {
                    println!("读取失败: {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("连接失败: {}", e);
            match e {
                ModbusConnectError::TcpError(tcp_e) => {
                    println!("TCP错误详情: {:?}", tcp_e);
                },
                ModbusConnectError::RtuError(rtu_e) => {
                    println!("RTU错误详情: {:?}", rtu_e);
                },
                ModbusConnectError::BothFailed(tcp_e, rtu_e) => {
                    println!("TCP错误详情: {:?}", tcp_e);
                    println!("RTU错误详情: {:?}", rtu_e);
                }
            }
        }
    }
}
