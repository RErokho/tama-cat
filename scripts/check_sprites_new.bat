@echo off
setlocal
cd /d "%~dp0\.."
echo === Desktop Cat: check NEW sprites ===
python scripts\check_sprites.py --new %*
exit /b %ERRORLEVEL%
