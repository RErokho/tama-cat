@echo off
setlocal
cd /d "%~dp0"

echo === Desktop Cat: build (release) ===
cargo build -p desktop-cat --release %*
set ERR=%ERRORLEVEL%
if %ERR% neq 0 exit /b %ERR%

echo.
echo OK: target\release\desktop-cat.exe
exit /b 0
