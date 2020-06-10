# Ultimate Music Replacement Mod

A Skyline plugin for replacing stream files (music, movies) with files of arbitrary size.

Mod files go in the following location:
```
/atmosphere/contents/01006A800016E000/romfs/stream/
```

For example, you can replace the menu music by placing a nus3audio file here:
```
/atmosphere/contents/01006A800016E000/romfs/stream/sound/bgm/bgm_crs2_01_menu.nus3audio
```

## Custom Atmosphère Configuration
To allow this plugin to function, you need to have the below set as so for `system_settings.ini` under `/atmosphere/config/`. If you do not have that config file under that directory, create it and paste the following code block contents into it.
The next Atmosphère release will not need this set manually.
```
[ro]
ease_nro_restriction = u8!0x1
```
