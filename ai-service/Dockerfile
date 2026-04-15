FROM python:3.11-slim AS runtime

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

RUN addgroup --system app && adduser --system --ingroup app app
WORKDIR /srv

COPY ai-service/pyproject.toml /srv/pyproject.toml
COPY ai-service/README.md /srv/README.md
COPY ai-service/src /srv/src
RUN pip install --no-cache-dir --upgrade pip && pip install --no-cache-dir .

USER app
EXPOSE 8001
HEALTHCHECK --interval=15s --timeout=3s --retries=5 CMD wget --spider -q http://127.0.0.1:8001/health/live || exit 1
CMD ["uvicorn", "main:app", "--app-dir", "/srv/src", "--host", "0.0.0.0", "--port", "8001"]
