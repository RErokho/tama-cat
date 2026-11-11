@echo off
setlocal
cd /d "%~dp0\.."

echo === Desktop Cat: check sprites ===
python scripts\check_sprites.py %*
set ERR=%ERRORLEVEL%

if %ERR% neq 0 (
  echo.
  echo FAILED — see issues above. Re-render bad frames, then:
  echo   python scripts\process_sprite_frame.py raw.jpg -o assets\sprites\cat\clean\STEM.png
  echo   python scripts\normalize_sprite_sizes.py
  echo   scripts\check_sprites.bat
  exit /b %ERR%
)

echo.
echo OK
exit /b 0
