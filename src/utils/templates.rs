pub fn get_ac_server_docker_compose(
    server_port_tcp: u16,
    server_port_udp: u16,
    http_port: u16,
) -> String {
    format!(
        r#"
services:
  ac_server:
    image: ubuntu:20.04
    volumes:
      - ./ac_files:/data
    ports:
      - "{0}:{0}/tcp"
      - "{1}:{1}/udp"
      - "{2}:{2}/tcp"
    working_dir: /data
    command: ./acServer
"#,
        server_port_tcp, server_port_udp, http_port
    )
}
