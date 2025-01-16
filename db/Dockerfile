# Use the official PostgreSQL image from the Docker Hub
FROM postgres:17.2

# hostname host.docker.internal
# Set environment variables for the PostgreSQL database
ENV POSTGRES_DB=${POSTGRES_DB}
ENV POSTGRES_USER=${POSTGRES_USER}
ENV POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
ENV PGDATA=${PGDATA}
# Expose the PostgreSQL port
EXPOSE 5432