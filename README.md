# tilemap-sandbox

[Japanese](README_ja.md)

![image](https://github.com/user-attachments/assets/2bbd5937-ccb6-4263-9c7d-7d4177f581b5)

## Overview

**tilemap-sandbox** is a tile-based 2D game development framework built using Rust and the Godot game engine. This project aims to construct a 2D tile-based world featuring dynamically editable tiles, blocks, and entities.

## Features

The primary features are as follows:

* **Separation of Concerns** : A well-structured design leveraging the high flexibility of the Godot game engine and the execution efficiency and type safety provided by Rust.
* **Efficient Data Management** : Efficient handling of tiles, blocks, and entities through the use of spatial indexing.
* **Flexible Extensibility** : Highly customizable event loop management for tasks such as procedural world generation, player movement, and animal AI behavior control.

## Extensibility

The following elements can be extended:

* New tiles, blocks, entities, and items
* New event loops
* New API endpoints for both Godot and Rust

## Project Structure and Extensibility

This project is composed of a single Godot project and two Rust crates:

* `/` : The Godot project, responsible for rendering and handling input events, and serves as the entry point of the application.
* `/native-core` : Contains core systems such as data flow and views.
* `/native-main` : Implements user-defined features and exposes APIs to the Godot engine.

