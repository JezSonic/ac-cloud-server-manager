# Assetto Corsa Cloud Server Manager

Assetto Corsa Cloud Server Manager is a dedicated application designed for remote management of Assetto Corsa dedicated servers. Built with Rust and featuring a modern user interface powered by QML and Qt, it provides a robust and efficient solution for server administrators to oversee their cloud-based racing environments.

## Description

The project aims to simplify the complexities of managing multiple Assetto Corsa server instances. By leveraging SSH and Docker, it allows for seamless remote control, content management, and real-time monitoring. The application bridge the gap between low-level server operations and a user-friendly management experience, ensuring high performance through Rust's safety and speed.

## Features

- Instance Management: Support for managing multiple Assetto Corsa server instances simultaneously.
- Remote Communication: Secure SSH and SFTP integration for remote command execution and file management.
- Docker Integration: Full support for managing server containers via the Docker API.
- Content Browsing:
  - Car List Management: Fetching car lists including skins and thumbnail previews.
  - Track Management: Support for various track layouts and previews.
- Real-time Monitoring: Live tracking of CPU usage, RAM consumption, network activity, and player statistics.
- Profile System: Secure storage and management of connection profiles for different servers.
- Modern Interface: Responsive GUI built with QML using the cxx-qt integration.

## Technology Stack

- Backend: Rust programming language for core logic and safety.
- Frontend: QML and Qt 6 for the user interface.
- Integration: cxx-qt for seamless communication between Rust and Qt/QML.
- Networking and Containers: ssh2 for remote access and bollard for Docker interaction.
- Compression and Serialization: tar, flate2 for content handling, and serde for data processing.
- Asynchronous Runtime: tokio for non-blocking operations and high performance.

## Project Structure

- src/core/: Contains core application logic, error handling, and profile management.
- src/gui/: Implementation of QML integration and UI-related logic.
- src/net/: Network modules covering SSH, Docker, telemetry, and content fetching.
- src/utils/: Helper utilities, system commands, and SFTP/Docker helpers.
- qml/: QML source files for views and components.
- locales/: Translation files for the user interface.

## Build Instructions

To build the project, the following dependencies are required:
- Rust (Cargo)
- Qt 6
- CMake
- OpenSSL development libraries

Execute the following command in the project root:

```bash
cargo build
```
