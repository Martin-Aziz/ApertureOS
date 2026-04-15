FROM node:20-alpine

WORKDIR /srv
RUN corepack enable

COPY frontend/package.json /srv/package.json
RUN pnpm install --frozen-lockfile=false

COPY frontend /srv

EXPOSE 5173
CMD ["pnpm", "dev", "--host", "0.0.0.0", "--port", "5173"]
