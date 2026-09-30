# Alexa bridge sidecar.
#
# This is the only Python image in the repository. It exists because Amazon's
# request signing and skill request model are implemented in the `ask-sdk`
# Python SDK and have no maintained Rust equivalent.
FROM python:3.13-slim

# git: one dependency is pinned to a VCS commit (see pyproject.toml's note on
# oscrypto and OpenSSL 3). Removed again in the same layer to keep it out of
# the final image.
WORKDIR /app

COPY pyproject.toml README.md ./
COPY src ./src

RUN apt-get update -qq \
 && apt-get install -y --no-install-recommends git \
 && pip install --no-cache-dir -q . \
 && apt-get purge -y --auto-remove git \
 && rm -rf /var/lib/apt/lists/*

# Runs unprivileged: this process is the one thing in the deployment exposed
# to the internet through the Cloudflare Tunnel.
RUN useradd --create-home --shell /usr/sbin/nologin bridge
USER bridge

EXPOSE 8081

# Two workers is ample for a household, and a short timeout keeps a stuck
# request from holding a worker past the window Alexa waits.
CMD ["gunicorn", "--bind", "0.0.0.0:8081", "--workers", "2", "--timeout", "15", \
     "--access-logfile", "-", "alexa_bridge.app:create_app()"]
