# Meal

Trying out OpenMLS + Rust + WebAssembly + SolidJS

# Next Goals

- [x] Use compression on production server when serving client files

- [x] Automate deployment after integration
  - [x] ~~Use [containrrr/watchtower](https://github.com/containrrr/watchtower)~~ Write own deployment system [tugboat](./tugboat/) ([Why?](./tugboat/README.md#why))
  - [x] Use bearer token to start update only on request
  - [x] Create github action that listens to new image publish and makes request to update server
- [x] Improve integration speed by ~~trying to use a base image~~ using multi state build and different targets that has dependencies installed
- Optimizations
  - A lot of work and not worth it at the moment
  - Continuously profile size
  - Record size changes over time to detect degradations
- [ ] [Push notifications](https://github.com/inKibra/tauri-plugins?tab=readme-ov-file) on iOS

# Requirements

- Fun to use
- End to end encrypted messages
- Encryption at rest
  - Hard to impossible in the browser
- Secure storage of encryption key
  - Either by user input or storage like secure OS level storage
  - Required to have encryption at rest
  - Not possible in the browser. Users would need to manually enter the key every time
