# Project name (optional, useful for prefixing container names)
COMPOSE = docker compose

.PHONY: all up build down restart logs clean fclean

# Default behavior when running just "make"
all: build


# Force a rebuild and restart the containers
build:
	$(COMPOSE) up --build -d


# Start the containers in the background (detached)
up:
	$(COMPOSE) up -d

# Stop and remove the containers and networks
down:
	$(COMPOSE) down

# Restart the entire stack
restart: down up

# Show live logs (Ctrl+C to exit)
logs:
	$(COMPOSE) logs -f

# Remove stopped containers and unused networks
clean:
	$(COMPOSE) down --remove-orphans

# Full cleanup: remove containers, networks AND volumes (WARNING: deletes the database!)
fclean:
	$(COMPOSE) down -v --remove-orphans