; Communication API with Telekinesis.DLL
; DO NOT USE DIRECTLY, use Tele_Devices instead to respect user settings

ScriptName Tele_Api hidden

Bool Function Loaded() global native
Bool Function Cmd(String cmd) Global Native
Bool Function Cmd_1(String cmd, String arg0) Global Native
Bool Function Cmd_2(String cmd, String arg0, String arg1) Global Native
String Function Qry_Str(String qry) Global Native
String Function Qry_Str_1(String qry, String arg0) Global Native
String[] Function Qry_Lst(String qry) Global Native
String[] Function Qry_Lst_1(String qry, String arg0) Global Native
Bool Function Qry_Bool(String qry) Global Native
Bool Function Qry_Bool_1(String qry, String arg0) Global Native
Int Function Tele_Control(String actuator, Int speed, Float time_sec, String pattern, String[] events) Global Native
Int Function Tele_Update(Int handle, Int speed) Global Native
Bool Function Tele_Stop(Int handle) Global Native
