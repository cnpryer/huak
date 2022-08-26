.PHONY: help clean lint fmt mt-check test pre-commit bench

help:
	@echo ""
	@echo "Use 'make <command>'"
	@echo ""
	@echo "commands"
	@echo "  .venv			    create venv and install dependencies"
	@echo "  clean				remove cleanable files"
	@echo "  lint				run flake8 linting"
	@cheo "  lint-types			run mypy check"
	@echo "  fmt				run formaters"
	@echo "  fmt-check			run formatting check"
	@echo "  test				run all tests"
	@echo "  pre-commit			run pre-commit standardization"
	@echo ""
	@echo "Check the Makefile to know exactly what each target is doing."

.venv:
	@python -m venv .venv
	@poetry install
	@pre-commit install

clean:
	-@rm -rf .venv
	-@rm -fr `find . -name __pycache__`
	-@rm -rf .pytest_cache
	-@rm -rf .mypy_cache

lint: .venv
	@poetry run flake8 \
		src \
		tests

lint-types: .venv
	@poetry run mypy \
		src \
		tests

fmt: .venv
	@poetry run isort .
	@poetry run black .

fmt-check: .venv
	@poetry run isort . --check
	@poetry run black . --check

test: .venv
	@poetry run pytest

pre-commit: test fmt-check lint lint-types
