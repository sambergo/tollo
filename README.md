# Töllö - A Modern IPTV Player

![Tollo Screenshot](placeholder.png)

Töllö is a modern, cross-platform desktop application for managing and watching IPTV playlists. It's built with a focus on performance, a clean user interface, and a robust set of features.

## ✨ Features

*   **Playlist Management**: Easily add, organize, and switch between multiple M3U playlists.
*   **Channel Browser**: A fast and intuitive interface for browsing through thousands of channels.
*   **Fuzzy Search**: Instantly find channels, groups, or anything else with a powerful fuzzy search.
*   **Video Playback**: Integrated video player with support for various stream formats.
*   **Favorites & History**: Keep track of your favorite channels and recently watched streams.
*   **Image Caching**: Caches channel logos for faster loading and offline availability.
*   **Customizable Settings**: Tweak the application to your liking.

## 🛠️ Tech Stack

*   **Frontend**: [React](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/), [Zustand](https://zustand-demo.pmnd.rs/)
*   **Backend**: [Rust](https://www.rust-lang.org/), [Tauri](https://tauri.app/)
*   **Database**: [SQLite](https://www.sqlite.org/index.html)

## 🚀 Getting Started

Follow these instructions to get a local copy up and running for development and testing.

### Prerequisites

*   [Node.js](https://nodejs.org/) (which includes npm)
*   [pnpm](https://pnpm.io/installation)
*   [Rust](https://www.rust-lang.org/tools/install)

### Installation

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/your-username/tollo.git
    cd tollo
    ```

2.  **Install frontend dependencies:**
    ```sh
    pnpm install
    ```

3.  **Build Rust dependencies:**
    This step is not strictly necessary before running the dev server, as `tauri dev` will handle it, but it can be useful to run it once to fetch and build all Rust crates.
    ```sh
    cd src-tauri
    cargo build
    cd ..
    ```

### Running the Application

To run the application in development mode, which provides hot-reloading for the frontend:

```sh
pnpm tauri dev
```

## 📦 Building for Production

To build the application for production, which will create a standalone executable for your platform:

```sh
pnpm tauri build
```

The compiled application will be located in `src-tauri/target/release/bundle/`.

## 📂 Project Structure

The project is organized into two main parts:

*   `src/`: Contains the React/TypeScript frontend code.
*   `src-tauri/`: Contains the Rust backend code, which handles all the core logic, database interactions, and native OS integrations.

```
.
├── src/                # Frontend source
│   ├── components/
│   ├── stores/
│   └── main.tsx
├── src-tauri/          # Backend source (Rust)
│   ├── src/
│   │   ├── database.rs
│   │   ├── m3u_parser.rs
│   │   └── main.rs
│   └── Cargo.toml
└── package.json
```

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
