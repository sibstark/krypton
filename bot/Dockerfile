FROM python:3.13.1-slim
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
WORKDIR /app
COPY requirements.txt .
RUN pip install uv
RUN uv pip install -r requirements.txt
COPY . .