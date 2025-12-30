## Local Development

1. Clone the repository:
   ```sh
   git clone https://gitee.com/cm_14/cdu_workbenrch_for_python.git
   cd cdu_workbenrch_for_python
   ```

2. Create a virtual environment and install dependencies:
   ```sh
   python -m venv venv
   source venv/bin/activate  # On Windows use `venv\Scripts\activate`
   pip install -r requirements.txt
   ```

3. Run the application:
   ```sh
   python run.py
   # or
   hypercorn run:app --bind 0.0.0.0:5000
   ```

## Build and Run with Podman
1. Install Podman:  
   ```sh
   sudo apt -y install podman
   ```
   _Follow the instructions for your operating system on the [Podman installation page](https://podman.io/getting-started/installation)._


1. Build the Podman image:
   ```sh
   podman build --arch arm64 -t cdu-workbench .
   ```
2. 
   ```sh
   sudo chmod 666 /dev/com4
   ```

2. Run the container:
   ```sh
   podman run -d \
   -p 5000:5000 -p 1161:1161 -p 5020:5020 \
   --device /dev/com4:/dev/com4 \
   -v ~/code/cdu_workbenrch_for_python/app/:/app/app \
   -v ~/code/cdu_workbenrch_for_python/logs/:/app/logs \
   -v ~/code/cdu_workbenrch_for_python/static_resources/:/app/static_resources \
   --name cdu cdu-workbench:1.0.11
   ```

3. Access the application:
   - Web UI: `http://<host_ip>:5000`
   - Modbus TCP Server: `http://<host_ip>:5020`
   - SNMP Server: `http://<host_ip>:1161`


_note: The Modbus TCP Server and SNMP Server are running in the container. You can access them from the host machine._

## Reference
- [Podman](./docs/podman.md)


1. 修改冷备轮询时间后重新开始计时 （倒计时）
2. 自动切换自动模式时间未生效
3. Cv 告警未记录
4. 时区选择


flow: -2 -1
