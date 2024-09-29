# Meal

Trying out OpenMLS + Rust + WebAssembly + SolidJS

# Next Goals

- Use compression on production server when serving client files
- Automate deployment after integration
  - Use [containrrr/watchtower](https://github.com/containrrr/watchtower)
  - Use bearer token to start update only on request
  - Create github action that listens to new image publish and makes request to update server
- Improve integration speed by trying to use a base image that has dependencies installed
- Optimizations
  - Continuously profile size
  - Record size changes over time to detect degredations
