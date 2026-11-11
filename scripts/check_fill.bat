@echo off
setlocal
cd /d "%~dp0\.."
echo === Desktop Cat: fill / mass profile + check ===
python scripts\build_scale_profile.py
if errorlevel 1 exit /b 1
python scripts\check_sprites.py --rebuild-profile %*
exit /b %ERRORLEVEL%
