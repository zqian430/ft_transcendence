# Project name (optional, useful for prefixing container names)
COMPOSE = docker compose
HOOKS = .githooks

.PHONY: all hooks build up down restart logs clean fclean

# Default behavior when running just "make"
all: hooks build

# Set up local Git hooks and auto-setup remote tracking branches
hooks:
	@git config core.hooksPath $(HOOKS) 2>/dev/null || true
	@git config push.autoSetupRemote true 2>/dev/null || true
	@chmod +x $(HOOKS)/* 2>/dev/null || true

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