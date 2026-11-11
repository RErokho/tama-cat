@echo off
setlocal
cd /d "%~dp0"

echo === Desktop Cat: run (debug) ===
cargo run -p desktop-cat %*
exit /b %ERRORLEVEL%
