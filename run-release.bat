@echo off
setlocal
cd /d "%~dp0"

if not exist "target\release\desktop-cat.exe" (
  echo Release exe not found — building...
  call "%~dp0build-release.bat"
  if errorlevel 1 exit /b 1
)

echo === Desktop Cat: run release exe ===
start "" "%~dp0target\release\desktop-cat.exe"
exit /b 0
