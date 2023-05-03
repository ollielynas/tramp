


@RD /S /Q ".\releases"

FOR %%A IN ("widows-x64", "windows-x32", "mac", "linux") DO if not exist %%A (mkdir ".\releases\"%%A)