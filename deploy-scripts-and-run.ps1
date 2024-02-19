Copy-Item -v "contrib\Distribution\SKSE\Plugins\*.dll"  "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\"
Copy-Item -v "contrib\Distribution\Scripts\*.pex" "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\Scripts\"
Copy-Item -v "contrib\Distribution\Source\Scripts\*.psc" "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\Source\Scripts\"
Get-ChildItem -Path "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\Telekinesis\Patterns\" -Include *.* -File -Recurse | ForEach-Object { $_.Delete()}
New-Item -ItemType Directory -Path "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\Telekinesis\Patterns\" -Force
Copy-Item -v "contrib\Distribution\SKSE\Plugins\Telekinesis\Patterns\*.funscript"  "C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition\Data\SKSE\Plugins\Telekinesis\Patterns\"
