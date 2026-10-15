@echo off
rem Launch F/A-18 Hornet 2.0 with the Systemless build beside this file.
rem Drag your game archive onto this file, or run:
rem   run-hornet.cmd "F-A-18 Hornet 2.0.toast"
if "%~1"=="" (
    echo Drag your game archive onto run-hornet.cmd, or run: run-hornet.cmd ARCHIVE
    pause
    exit /b 2
)
set "SYSTEMLESS_LOAD_EXECUTABLE=%~2"
if "%SYSTEMLESS_LOAD_EXECUTABLE%"=="" set "SYSTEMLESS_LOAD_EXECUTABLE=Hornet"
"%~dp0systemless.exe" --prefer-68k "%~1"
