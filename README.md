# 🦀 Rust Vulkan Experiments

A simple experimentation project to discover Vulkan with Rust and draw a triangle.

## Description

This project was created to explore the Vulkan API in Rust, building upon previous experience with Vulkan in C++. The main objective is to understand how Vulkan works in the Rust ecosystem and see the differences compared to C++ implementation.

## Features

- Complete Vulkan instance initialization
- Physical device selection
- Rendering surface creation with Winit
- Swapchain configuration
- Rendering pipeline with vertex and fragment shaders
- Simple colored triangle rendering

## Project Structure

```
src/
├── main.rs              # Main entry point
├── lib.rs               # Main module
├── window/              # Window management
├── vulkan/              # Vulkan wrappers
│   ├── instance.rs
│   ├── device.rs
│   ├── swapchain.rs
│   └── ...
├── pipeline/            # Rendering pipeline
└── renderer/            # Rendering logic
```

## Dependencies

- `ash` : Rust bindings for Vulkan
- `winit` : Window and event management
- `anyhow` : Error handling

## Build and Run

```bash
cargo run
```

## Conclusion

This project was mainly an exploration to see "what Vulkan looks like" with Rust. While the experience was instructive, I found that the significant amount of `unsafe` code required for Vulkan makes this approach quite cumbersome for real projects.

In my opinion, for future graphics rendering projects, **WGPU** is much more appealing because:

- It offers a safer and more idiomatic API in Rust
- It automatically handles the complexity of Vulkan/DirectX/Metal
- It significantly reduces the amount of boilerplate code
- It's more maintainable and less error-prone

While Vulkan remains an excellent choice for specific use cases requiring fine performance control, I believe that for most Rust graphics applications, WGPU offers a much better balance between simplicity and power.

In any case, I think that's the direction I'm going to choose.
