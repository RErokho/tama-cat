@echo off
setlocal
cd /d "%~dp0"

echo === Desktop Cat: build (debug) ===
cargo build -p desktop-cat %*
set ERR=%ERRORLEVEL%
if %ERR% neq 0 exit /b %ERR%

echo.
echo OK: target\debug\desktop-cat.exe
exit /b 0
