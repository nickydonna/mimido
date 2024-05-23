# syntax = docker/dockerfile:1

# Adjust NODE_VERSION as desired
ARG NODE_VERSION=18.19.1
FROM node:${NODE_VERSION}-slim as base

LABEL fly_launch_runtime="SvelteKit"

# SvelteKit app lives here
WORKDIR /app

# Set production environment
ENV NODE_ENV="production"

# Install pnpm
ARG PNPM_VERSION=8.15.4
RUN npm install -g pnpm@$PNPM_VERSION

# Throw-away build stage to reduce size of final image
FROM base as build

# Install packages needed to build node modules
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y build-essential node-gyp pkg-config python-is-python3

# Install node modules
COPY --link .npmrc package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile --prod=false

# Copy application code
COPY --link . .

RUN pnpm dlx prisma generate

# Build application
RUN --mount=type=secret,id=PUBLIC_SENTRY_DNS \
    PUBLIC_SENTRY_DNS="$(cat /run/secrets/PUBLIC_SENTRY_DNS)" \
    pnpm run build


# Remove development dependencies
RUN pnpm prune --prod

# Final stage for app image
FROM base

# for debian/ubuntu-based images
RUN apt-get update -y && apt-get install -y ca-certificates fuse3 sqlite3

COPY --from=flyio/litefs:0.5 /usr/local/bin/litefs /usr/local/bin/litefs
ADD etc/litefs.yml /etc/litefs.yml

# Copy built application
COPY --from=build /app/build /app/build
COPY --from=build /app/prisma /app/prisma
COPY --from=build /app/node_modules /app/node_modules
COPY --from=build /app/package.json /app

# Start the server by default, this can be overwritten at runtime
# EXPOSE 3000
# CMD [ "pnpm", "run", "start" ]

ENTRYPOINT litefs mount
