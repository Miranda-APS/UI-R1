@echo off
chcp 65001 >nul
echo 🧠 Prometeo Semantic Agent
echo ==========================

if "%~1"=="" (
    echo.
    echo Uso: run_agent.bat [task] [iterazioni]
    echo.
    echo Task disponibili:
    echo   analyze    - Analisi e riparazione connessioni
    echo   create     - Creazione nuovi concetti
    echo   explore    - Esplorazione libera (default)
    echo.
    echo Esempi:
    echo   run_agent.bat analyze 30
    echo   run_agent.bat create 20
    echo   run_agent.bat explore 50 --dry-run
    goto :end
)

set TASK=%~1
set ITER=%~2
if "%ITER%"=="" set ITER=30

if "%TASK%"=="analyze" (
    python agent\loop.py --task steering\tasks\analyze_and_repair.json -i %ITER%
) else if "%TASK%"=="create" (
    python agent\loop.py --task steering\tasks\create_concepts.json -i %ITER%
) else if "%TASK%"=="explore" (
    python agent\loop.py -i %ITER%
) else (
    echo Task sconosciuto: %TASK%
    echo Usa: analyze, create, o explore
)

:end
