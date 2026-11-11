GOLINES ?= $(shell command -v golines 2>/dev/null)
WAILS ?= $(shell command -v wails 2>/dev/null)
GO_FILES := $(shell rg --files -g '*.go')
FRONTEND_DIR := frontend
NPM ?= npm

.PHONY: format format-go format-frontend frontend-install frontend-build frontend-dev wails-dev wails-build

format: format-go format-frontend

format-go:
ifndef GOLINES
	$(error golines binary not found. Install with `go install github.com/segmentio/golines@latest`)
endif
ifeq ($(strip $(GO_FILES)),)
	@echo "No Go files to format"
else
	@echo "Formatting Go files with golines (max len 120)"
	@$(GOLINES) -w -m 120 $(GO_FILES)
endif

format-frontend:
	@echo "Formatting frontend with Prettier"
	@cd $(FRONTEND_DIR) && $(NPM) run --silent format

frontend-install:
	@cd $(FRONTEND_DIR) && $(NPM) install

frontend-build:
	@cd $(FRONTEND_DIR) && $(NPM) run build

frontend-dev:
	@cd $(FRONTEND_DIR) && $(NPM) run dev

wails-dev:
ifndef WAILS
	$(error wails binary not found. Install with `go install github.com/wailsapp/wails/v2/cmd/wails@latest`)
endif
	@$(WAILS) dev

wails-build:
ifndef WAILS
	$(error wails binary not found. Install with `go install github.com/wailsapp/wails/v2/cmd/wails@latest`)
endif
	@$(WAILS) build -clean
