#!/bin/bash
mkdir -p $PWD/.vscode-web
docker run --rm -it -p 127.0.0.1:8080:8080 \
  -v "$PWD:/home/coder/project" \
  -v "$PWD/.config:/home/coder/.config" \
  -v "$PWD/.vscode-web:/home/coder/.local/share/code-server" \
  -v "$PWD/.vscode/extensions-web.json:/home/coder/project/.vscode/extensions.json" \
  -u "$(id -u):$(id -g)" \
  --name code \
  j0rsa/rust-code-server:latest /home/coder/project