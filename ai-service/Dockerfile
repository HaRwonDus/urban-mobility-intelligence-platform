FROM python:3.12-slim

WORKDIR /app

ENV POETRY_VERSION=1.8.5 \
    POETRY_VIRTUALENVS_CREATE=false \
    POETRY_NO_INTERACTION=1

RUN pip install --no-cache-dir "poetry==$POETRY_VERSION"

COPY pyproject.toml .
RUN poetry install --only main --no-ansi --no-root

COPY app ./app

EXPOSE 8010
CMD ["poetry", "run", "uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8010"]
