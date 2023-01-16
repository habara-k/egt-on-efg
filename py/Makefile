install:
	python3 -m pip install -r requirements.txt
format:
	python3 -m black .
	python3 -m isort .
check:
	python3 -m black . --check --diff
	python3 -m isort . --check --diff
	python3 -m mypy .
