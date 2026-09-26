FROM python:3.12-slim
WORKDIR /app
RUN pip install uv -q
COPY pyproject.toml uv.lock* ./
RUN uv sync --frozen -q
COPY . .
