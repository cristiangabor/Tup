FROM mcr.microsoft.com/devcontainers/base:bookworm

ARG USERNAME=vscode

USER ${USERNAME}

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal \
    && /home/${USERNAME}/.cargo/bin/rustup component add rustfmt clippy

ENV PATH="/home/${USERNAME}/.cargo/bin:${PATH}"

WORKDIR /workspaces/Tup