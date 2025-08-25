@echo off
echo Building Open Reverb for Windows...

REM Build common library
echo Building common library...
cd open-reverb-common
cargo build --release
if %ERRORLEVEL% neq 0 goto error
cd ..

REM Build server
echo Building server...
cd open-reverb-server
cargo build --release
if %ERRORLEVEL% neq 0 goto error
cd ..

REM Build client
echo Building client...
cd open-reverb-client
cargo build --release
if %ERRORLEVEL% neq 0 goto error
cd ..

REM Create dist directory
if not exist "dist\windows" mkdir dist\windows

REM Copy binaries to dist
copy target\release\open-reverb-server.exe dist\windows\
copy target\release\open-reverb-client.exe dist\windows\

echo Build completed successfully!
echo Binaries are available in the dist\windows directory
goto :eof

:error
echo Build failed with error code %ERRORLEVEL%
exit /b %ERRORLEVEL%