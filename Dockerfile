from ubuntu:jammy

WORKDIR /workspace
COPY . /workspace

# Install Rust
RUN apt-get update && \
    apt-get install curl -y && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

# Install VULKAN
RUN apt-get update && apt-get install -y \
    vulkan-tools \
    libvulkan1 \
    mesa-vulkan-drivers \
    mesa-utils \
    libgl1-mesa-glx \
    libegl1 \
    libegl1-mesa \
    libgl1-mesa-dri \
    libglu1-mesa \
    libxrandr2 libxinerama1 libxcursor1 libxi6 \
    build-essential \
    libxkbcommon-x11-0

# Build the Custom Rerun Visualization SDK
RUN . "$HOME/.cargo/env" && \
    cargo build -p custom_callback_mod --bin custom_callback_viewer_mod --release

ENTRYPOINT [ "bash" ]