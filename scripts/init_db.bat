@echo off
:: This command prevents the terminal from printing every single command it runs, keeping it clean.

setlocal
:: This ensures variables we set here don't mess up your main computer settings.

:: --- CONFIGURATION ---
set DB_USER=postgres
set DB_PASSWORD=password
set DB_NAME=fallguard
set DB_PORT=5432
set CONTAINER_NAME=fallguard_db

:: --- STEP 1: CHECK DOCKER ---
echo [INFO] Checking if Docker is running...
docker info >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Docker is not running! 
    echo         Please open "Docker Desktop" app and try again.
    exit /b 1
)

:: --- STEP 2: CHECK EXISTING CONTAINER ---
:: We check if a container named 'fallguard_db' already exists.
docker ps -a --filter "name=%CONTAINER_NAME%" --format "{{.Names}}" | findstr "%CONTAINER_NAME%" >nul
IF %ERRORLEVEL% EQU 0 (
    echo [INFO] Container '%CONTAINER_NAME%' already exists. Starting it...
    docker start %CONTAINER_NAME%
) ELSE (
    echo [INFO] Creating new database container...
    :: This command downloads Postgres and runs it
    docker run --name %CONTAINER_NAME% ^
      -e POSTGRES_USER=%DB_USER% ^
      -e POSTGRES_PASSWORD=%DB_PASSWORD% ^
      -e POSTGRES_DB=%DB_NAME% ^
      -p %DB_PORT%:5432 ^
      -d postgres:15-alpine
)

:: --- STEP 3: WAIT FOR DATABASE ---
echo [INFO] Waiting for database to start up...
timeout /t 5 /nobreak >nul

echo [SUCCESS] Database is ready!
echo Connection String: postgres://%DB_USER%:%DB_PASSWORD%@localhost:%DB_PORT%/%DB_NAME%
endlocal