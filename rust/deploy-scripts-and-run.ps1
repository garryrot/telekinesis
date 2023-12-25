cp -v "..\contrib\Distribution\SKSE\Plugins\*.dll"  "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\"
cp -v "..\contrib\Distribution\Scripts\*.pex" "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\Scripts\";
Get-ChildItem -Path "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\Telekinesis\Patterns\" -Include *.* -File -Recurse | foreach { $_.Delete()}
cp -v "..\contrib\Distribution\SKSE\Plugins\Telekinesis\Patterns\*.funscript"  "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\Telekinesis\Patterns\"
