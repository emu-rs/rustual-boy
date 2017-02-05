# This script takes care of packaging the build artifacts that will go in the
# release zipfile

cd rustual-boy-cli

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"

# TODO Update this to package the right artifacts
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\rustual-boy.exe" '.\'
Copy-Item "$SRC_DIR\doc\*" '.\'
Copy-Item "$SRC_DIR\LICENSE-APACHE" '.\'
Copy-Item "$SRC_DIR\LICENSE-MIT" '.\'
Copy-Item "$SRC_DIR\LICENSE-THIRD-PARTY" '.\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR
