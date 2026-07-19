# FTP Server (Python)

This project is a simple Python FTP server using **uv** as the
environment and dependency manager.\
The `ftp_data/` directory contains the file tree exposed by the FTP
server.

## Requirements

-   Python **3.10+**
-   Linux, macOS, or Windows (WSL recommended)

## Install `uv`

`uv` is a fast Python package and environment manager.

### Linux / macOS

``` bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

Reload your shell if needed:

### Verify installation

``` bash
uv --version
```

## Project setup

Move to the project root directory:

``` bash
cd ftp-server
```

Dependencies are defined in `pyproject.toml` and locked in `uv.lock`.\
No manual installation is required.

## Run the server

``` bash
uv run main.py
```

`uv` will: 1. Create an isolated virtual environment 2. Install
dependencies if necessary 3. Run `main.py`

## Project structure

``` text
.
├── ftp_data/        # FTP root directory
├── main.py          # Application entry point
├── pyproject.toml   # Project configuration and dependencies
├── uv.lock          # Dependency lockfile
└── README.md
```

## Notes

-   `ftp_data/` is used as the FTP root directory
-   Restart the server to reflect filesystem changes
-   Prefer `uv run` over `python main.py` for reproducibility